// entrypoint for dll

#include <stdint.h>
#include <stdio.h>
#include <string.h>

#ifdef _WIN32

#include <windows.h>

#endif // _WIN32

typedef enum SlotTag {
    I32,
    I64,
    INative,
    F32,
    F64,
    Ref,
    Value,
    Uninit,
} SlotTag;

typedef union SlotData {
    int32_t i32_;
    int64_t i64_;
    int inative_;
    float f32_;
    double f64_;
    uint8_t *ptr_;
} SlotData;

typedef struct Slot {
    SlotTag tag;
    SlotData data;
} Slot;

typedef enum NativeState {
    Ok,
    NoFunc,
} NativeState;


#define NEXT_ARG(T, var) { (var) = *(T*)(args + offset); offset += sizeof(T); }

#ifdef _WIN32
_declspec(dllexport)
#endif // _WIN32
    NativeState native_bridge(const char *fname, const uint8_t *args, Slot *ret) {
    int offset = 0;
    if (strcmp(fname, "putchar") == 0) {
        // TODO: utf-16 support after char is implemented
        int32_t v0;
        NEXT_ARG(int32_t, v0); 
        putchar(v0);
    } else if (strcmp(fname, "puti32") == 0) {
        int32_t v0;
        NEXT_ARG(int32_t, v0); 
        fprintf(stdout, "%d", v0);
    } else {
        return NoFunc;
    }
    return Ok;
}

#ifdef _WIN32

BOOL APIENTRY DllMain(HANDLE hModule, DWORD ul_reason_for_call,
                      LPVOID lpReserved) {
    switch (ul_reason_for_call) {
    case DLL_PROCESS_ATTACH:
        break;
    case DLL_THREAD_ATTACH:
        break;
    case DLL_THREAD_DETACH:
        break;
    case DLL_PROCESS_DETACH:
        break;
    }
    return TRUE;
}

#endif // _WIN32
