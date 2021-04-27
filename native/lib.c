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
    Ref,
    F,
    Uninit,
} SlotTag;

typedef union SlotData {
    int32_t i32_;
    int64_t i64_;
    int inative_;
    unsigned int unative_;
} SlotData;

typedef struct Slot {
    SlotTag tag;
    SlotData data;
} Slot;

typedef enum NativeState {
    Ok,
    NoFunc,
    WrongArgc,
    WrongArgTy,
} NativeState;

#ifdef _WIN32
_declspec(dllexport) NativeState _cdecl
#else
NativeState
#endif // _WIN32
    native_bridge(const char *fname, int32_t argc, const Slot *args,
                  Slot *ret) {
    if (strcmp(fname, "putchar") == 0) {
        if (argc != 1) {
            return WrongArgc;
        }
        switch (args[0].tag) {
        case I32:
            putchar(args[0].data.i32_);
            break;
        case INative:
            putchar(args[0].data.inative_);
            break;
        default:
            return WrongArgTy;
        }
    } else if (strcmp(fname, "puti32") == 0) {
        if (argc != 1) {
            return WrongArgc;
        }
        switch (args[0].tag) {
        case I32:
            fprintf(stdout, "%d", args[0].data.i32_);
            break;
        default:
            return WrongArgTy;
        }
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
