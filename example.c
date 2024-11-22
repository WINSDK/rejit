#include <stdio.h>
#include <stdbool.h>

extern bool matches(const char* input);

int main(int argc, char** argv) {
    if (argc < 2) {
        printf("Missing expression.\n");
        return 1;
    }

    if (matches(argv[1])) {
        printf("Expression '%s' matches.\n", argv[1]);
    } else {
        printf("Expression '%s' does NOT match.\n", argv[1]);
    }

    return 0;
}
