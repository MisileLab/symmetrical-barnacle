#include <stdio.h>
#include <pthread.h>
#include <stdlib.h>

// Flux print function - prints an i32 and returns it
int print(int value) {
    printf("%d\n", value);
    fflush(stdout);
    return value;
}

// Parallel execution support
typedef struct {
    int (*func)(int);  // Function pointer
    int arg;           // Argument
    int result;        // Result
} ParallelTask;

// Thread worker function
static void* worker_thread(void* arg) {
    ParallelTask* task = (ParallelTask*)arg;
    task->result = task->func(task->arg);
    return NULL;
}

// Execute two pure functions in parallel
// Returns results as: (result1 << 32) | result2
// For simplicity, we'll just return result1 for now
int parallel_exec_2(int (*f1)(int), int arg1, int (*f2)(int), int arg2) {
    ParallelTask task1 = {f1, arg1, 0};
    ParallelTask task2 = {f2, arg2, 0};

    pthread_t thread1, thread2;

    // Launch threads
    pthread_create(&thread1, NULL, worker_thread, &task1);
    pthread_create(&thread2, NULL, worker_thread, &task2);

    // Wait for completion
    pthread_join(thread1, NULL);
    pthread_join(thread2, NULL);

    // For demo, just return first result
    // In real implementation, would return both results
    return task1.result;
}

// Parallel map: apply function to range [start, end)
int parallel_map_range(int (*func)(int), int start, int end) {
    if (end - start <= 1) {
        return func(start);
    }

    int mid = (start + end) / 2;

    ParallelTask task1 = {func, start, 0};
    ParallelTask task2 = {func, mid, 0};

    pthread_t thread1, thread2;

    // Process first half
    pthread_create(&thread1, NULL, worker_thread, &task1);

    // Process second half recursively
    int result2 = parallel_map_range(func, mid, end);

    pthread_join(thread1, NULL);

    return task1.result + result2;
}
