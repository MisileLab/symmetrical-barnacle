#include <stdio.h>
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdatomic.h>

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

// Thread pool management
static atomic_int active_threads = 0;
static int max_threads = 0;

// Initialize thread pool (called once at startup)
static void init_thread_pool() {
    if (max_threads == 0) {
        max_threads = sysconf(_SC_NPROCESSORS_ONLN);  // Get CPU count
        if (max_threads <= 0) max_threads = 4;  // Fallback
        // Reserve some threads for system
        if (max_threads > 2) max_threads -= 1;
    }
}

// Thread worker function
static void* worker_thread(void* arg) {
    ParallelTask* task = (ParallelTask*)arg;
    task->result = task->func(task->arg);
    atomic_fetch_sub(&active_threads, 1);  // Decrement on exit
    return NULL;
}

// Execute two pure functions in parallel and return the SUM
// This is used for auto-parallelization of expressions like: f(x) + g(y)
// Both f and g execute in parallel, then results are added
// Thread count is limited to nproc to avoid thread explosion
int parallel_exec_2(int (*f1)(int), int arg1, int (*f2)(int), int arg2) {
    init_thread_pool();

    // Check if we can spawn threads (limit to max_threads)
    int current_threads = atomic_load(&active_threads);

    // If too many threads active, execute sequentially
    if (current_threads >= max_threads) {
        int r1 = f1(arg1);
        int r2 = f2(arg2);
        return r1 + r2;
    }

    ParallelTask task1 = {f1, arg1, 0};
    ParallelTask task2 = {f2, arg2, 0};

    pthread_t thread1, thread2;

    // Reserve thread slots
    atomic_fetch_add(&active_threads, 2);

    // Launch both functions in parallel
    pthread_create(&thread1, NULL, worker_thread, &task1);
    pthread_create(&thread2, NULL, worker_thread, &task2);

    // Wait for both to complete
    pthread_join(thread1, NULL);
    pthread_join(thread2, NULL);

    // Return the sum (for fib(n-1) + fib(n-2) pattern)
    return task1.result + task2.result;
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
