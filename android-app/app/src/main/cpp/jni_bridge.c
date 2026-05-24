/**
 * JNI bridge between Android Java/Kotlin and AstraShell Rust runtime
 * This is the critical interface layer that connects the Android app
 * to the native Rust runtime system.
 */

#include <jni.h>
#include <string.h>
#include <android/log.h>

#define LOG_TAG "AstraShell-JNI"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// Rust runtime API (C ABI)
extern int astra_runtime_init(const char* config_json);
extern int astra_runtime_start(void);
extern int astra_runtime_stop(void);
extern const char* astra_runtime_status(void);
extern int astra_runtime_exec(const char* cmd, const char* const* args, int arg_count);
extern const char* astra_runtime_exec_output(const char* cmd, const char* const* args, int arg_count);
extern int astra_overlay_mount(const char* lower, const char* upper, const char* work, const char* merged);
extern int astra_overlay_unmount(const char* merged);
extern const char* astra_check_capabilities(void);
extern int astra_create_userns(const char* uid_map, const char* gid_map);
extern int astra_avf_init(const char* config);
extern int astra_avf_start(int vm_id);
extern int astra_avf_stop(int vm_id);
extern const char* astra_avf_exec(int vm_id, const char* command);
extern int astra_oci_pull(const char* image_ref);
extern const char* astra_oci_list(void);
extern const char* astra_version(void);

// Runtime init
JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_runtimeInit(JNIEnv* env, jclass clazz, jstring config_json) {
    const char* config = (*env)->GetStringUTFChars(env, config_json, NULL);
    int result = astra_runtime_init(config);
    (*env)->ReleaseStringUTFChars(env, config_json, config);
    LOGI("runtimeInit: %d", result);
    return result == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_runtimeStart(JNIEnv* env, jclass clazz) {
    int result = astra_runtime_start();
    LOGI("runtimeStart: %d", result);
    return result == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_runtimeStop(JNIEnv* env, jclass clazz) {
    int result = astra_runtime_stop();
    LOGI("runtimeStop: %d", result);
    return result == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jstring JNICALL
Java_com_astrashell_bridge_NativeBridge_runtimeStatus(JNIEnv* env, jclass clazz) {
    const char* status = astra_runtime_status();
    return (*env)->NewStringUTF(env, status ? status : "unknown");
}

JNIEXPORT jint JNICALL
Java_com_astrashell_bridge_NativeBridge_runtimeExec(
    JNIEnv* env, jclass clazz, jstring command, jobjectArray args) {
    const char* cmd = (*env)->GetStringUTFChars(env, command, NULL);
    jsize count = args ? (*env)->GetArrayLength(env, args) : 0;

    const char** cargs = NULL;
    if (count > 0) {
        cargs = (const char**)malloc(sizeof(char*) * count);
        for (int i = 0; i < count; i++) {
            jstring s = (jstring)(*env)->GetObjectArrayElement(env, args, i);
            cargs[i] = (*env)->GetStringUTFChars(env, s, NULL);
        }
    }

    int result = astra_runtime_exec(cmd, cargs, count);

    if (cargs) {
        for (int i = 0; i < count; i++) {
            (*env)->ReleaseStringUTFChars(env, (jstring)(*env)->GetObjectArrayElement(env, args, i), cargs[i]);
        }
        free(cargs);
    }
    (*env)->ReleaseStringUTFChars(env, command, cmd);

    return result;
}

JNIEXPORT jstring JNICALL
Java_com_astrashell_bridge_NativeBridge_runtimeExecOutput(
    JNIEnv* env, jclass clazz, jstring command, jobjectArray args) {
    const char* cmd = (*env)->GetStringUTFChars(env, command, NULL);
    jsize count = args ? (*env)->GetArrayLength(env, args) : 0;

    const char** cargs = NULL;
    if (count > 0) {
        cargs = (const char**)malloc(sizeof(char*) * count);
        for (int i = 0; i < count; i++) {
            jstring s = (jstring)(*env)->GetObjectArrayElement(env, args, i);
            cargs[i] = (*env)->GetStringUTFChars(env, s, NULL);
        }
    }

    const char* output = astra_runtime_exec_output(cmd, cargs, count);

    if (cargs) {
        for (int i = 0; i < count; i++) {
            (*env)->ReleaseStringUTFChars(env, (jstring)(*env)->GetObjectArrayElement(env, args, i), cargs[i]);
        }
        free(cargs);
    }
    (*env)->ReleaseStringUTFChars(env, command, cmd);

    return (*env)->NewStringUTF(env, output ? output : "");
}

JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_overlayMount(
    JNIEnv* env, jclass clazz, jstring lower, jstring upper, jstring work, jstring merged) {
    const char* clower = (*env)->GetStringUTFChars(env, lower, NULL);
    const char* cupper = (*env)->GetStringUTFChars(env, upper, NULL);
    const char* cwork = (*env)->GetStringUTFChars(env, work, NULL);
    const char* cmerged = (*env)->GetStringUTFChars(env, merged, NULL);

    int result = astra_overlay_mount(clower, cupper, cwork, cmerged);

    (*env)->ReleaseStringUTFChars(env, lower, clower);
    (*env)->ReleaseStringUTFChars(env, upper, cupper);
    (*env)->ReleaseStringUTFChars(env, work, cwork);
    (*env)->ReleaseStringUTFChars(env, merged, cmerged);

    return result == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_overlayUnmount(JNIEnv* env, jclass clazz, jstring merged) {
    const char* cmerged = (*env)->GetStringUTFChars(env, merged, NULL);
    int result = astra_overlay_unmount(cmerged);
    (*env)->ReleaseStringUTFChars(env, merged, cmerged);
    return result == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jstring JNICALL
Java_com_astrashell_bridge_NativeBridge_checkCapabilities(JNIEnv* env, jclass clazz) {
    const char* caps = astra_check_capabilities();
    return (*env)->NewStringUTF(env, caps ? caps : "{}");
}

JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_createUserNamespace(
    JNIEnv* env, jclass clazz, jstring uid_map, jstring gid_map) {
    const char* cuid = uid_map ? (*env)->GetStringUTFChars(env, uid_map, NULL) : NULL;
    const char* cgid = gid_map ? (*env)->GetStringUTFChars(env, gid_map, NULL) : NULL;

    int result = astra_create_userns(cuid, cgid);

    if (cuid) (*env)->ReleaseStringUTFChars(env, uid_map, cuid);
    if (cgid) (*env)->ReleaseStringUTFChars(env, gid_map, cgid);

    return result == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jint JNICALL
Java_com_astrashell_bridge_NativeBridge_avfInit(JNIEnv* env, jclass clazz, jstring config) {
    const char* cconfig = (*env)->GetStringUTFChars(env, config, NULL);
    int vm_id = astra_avf_init(cconfig);
    (*env)->ReleaseStringUTFChars(env, config, cconfig);
    return vm_id;
}

JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_avfStart(JNIEnv* env, jclass clazz, jint vm_id) {
    return astra_avf_start(vm_id) == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_avfStop(JNIEnv* env, jclass clazz, jint vm_id) {
    return astra_avf_stop(vm_id) == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jstring JNICALL
Java_com_astrashell_bridge_NativeBridge_avfExec(JNIEnv* env, jclass clazz, jint vm_id, jstring command) {
    const char* cmd = (*env)->GetStringUTFChars(env, command, NULL);
    const char* output = astra_avf_exec(vm_id, cmd);
    (*env)->ReleaseStringUTFChars(env, command, cmd);
    return (*env)->NewStringUTF(env, output ? output : "");
}

JNIEXPORT jboolean JNICALL
Java_com_astrashell_bridge_NativeBridge_ociPull(JNIEnv* env, jclass clazz, jstring image_ref) {
    const char* ref = (*env)->GetStringUTFChars(env, image_ref, NULL);
    int result = astra_oci_pull(ref);
    (*env)->ReleaseStringUTFChars(env, image_ref, ref);
    return result == 0 ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT jstring JNICALL
Java_com_astrashell_bridge_NativeBridge_ociList(JNIEnv* env, jclass clazz) {
    const char* list = astra_oci_list();
    return (*env)->NewStringUTF(env, list ? list : "[]");
}

JNIEXPORT jstring JNICALL
Java_com_astrashell_bridge_NativeBridge_getVersion(JNIEnv* env, jclass clazz) {
    const char* version = astra_version();
    return (*env)->NewStringUTF(env, version ? version : "0.0.0");
}
