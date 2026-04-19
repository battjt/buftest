#include <locale.h>
#include <pthread.h>
#include <stdio.h>
#include <unistd.h>

// use 'long long [2]

void *tx(void *name)
{
    long long seq[2] = {0, 0};
    long long last = 0;
    long next = 0;
    long now;
    long start = time(NULL) - 1;
    while (1)
    {
        fwrite(&seq, 16, 1, stdout);
        seq[0]++;
        if (seq[0] % 10000 == 0 && (now = time(NULL)) > next)
        {
            fprintf(stderr, "%s TX: %'lld %'lld/s\n", (char *)name, seq[0] - last, seq[0] / (now - start));
            last = seq[0];
            next = now;
        }
    }
}
void *flush(void *name)
{
    while (1)
    {
        fflush(stdout);
        usleep(2000); // 2ms
    }
}
void *rx(void *name)
{
    long long seq[2] = {0, 0};
    long long last = 0;
    long next = 0;
    long now;
    long start = time(NULL) - 1;
    while (1)
    {
        fread(&seq, 16, 1, stdin);
        if (seq[0] % 10000 == 0 && (now = time(NULL)) > next)
        {
            fprintf(stderr, "%s RX: %'lld %'lld/s\n", (char *)name, seq[0] - last, seq[0] / (now - start));
            last = seq[0];
            next = now;
        }
    }
}

int main(int argc, char *argv[])
{
    setlocale(LC_NUMERIC, ""); // Use the system's default locale

    pthread_t tx_thread, rx_thread, flush_thread;
    pthread_create(&flush_thread, NULL, flush, argv[1]);
    pthread_create(&tx_thread, NULL, tx, argv[1]);
    pthread_create(&rx_thread, NULL, rx, argv[1]);
    pthread_join(tx_thread, NULL);
    pthread_join(rx_thread, NULL);
    return 0;
}