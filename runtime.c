#include <stdio.h>

// Flux print function - prints an i32 and returns it
int print(int value) {
    printf("%d\n", value);
    fflush(stdout);
    return value;
}
