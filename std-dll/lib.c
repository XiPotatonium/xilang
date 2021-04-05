// entrypoint for dll

#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <windows.h>

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

_declspec(dllexport) NativeState
    _cdecl native_bridge(const char *fname, int32_t argc, const Slot *args,
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
    } else {
        return WrongArgc;
    }
    return Ok;
}

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
