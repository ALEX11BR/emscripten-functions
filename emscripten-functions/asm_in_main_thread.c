#include <emscripten.h>

void asm_in_main_thread(char *script) {
    MAIN_THREAD_EM_ASM("eval(UTF8ToString($0))", script);
}

int asm_in_main_thread_int(char *script) {
    return MAIN_THREAD_EM_ASM_INT("eval(UTF8ToString($0))", script);
}

double asm_in_main_thread_double(char *script) {
    return MAIN_THREAD_EM_ASM_DOUBLE("eval(UTF8ToString($0))", script);
}
