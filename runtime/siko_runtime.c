#define _GNU_SOURCE
#include "runtime/siko_runtime.h"

#include <fcntl.h>
#include <pthread.h>
#include <stdint.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

int siko_pthread_join(uint64_t thread, void **retval) {
    return pthread_join((pthread_t)(uintptr_t)thread, retval);
}

static int siko_set_cloexec(int fd) {
    int flags = fcntl(fd, F_GETFD);
    if (flags < 0) {
        return -1;
    }
    return fcntl(fd, F_SETFD, flags | FD_CLOEXEC);
}

int siko_pipe_create(int pipefd[2]) {
#if defined(__linux__) && defined(O_CLOEXEC)
    return pipe2(pipefd, O_CLOEXEC);
#else
    if (pipe(pipefd) < 0) {
        return -1;
    }
    if (siko_set_cloexec(pipefd[0]) < 0) {
        close(pipefd[0]);
        close(pipefd[1]);
        return -1;
    }
    if (siko_set_cloexec(pipefd[1]) < 0) {
        close(pipefd[0]);
        close(pipefd[1]);
        return -1;
    }
    return 0;
#endif
}

int siko_fd_set_nonblocking(int fd, int nonblocking) {
    int flags = fcntl(fd, F_GETFL);
    if (flags < 0) {
        return -1;
    }
    if (nonblocking) {
        flags |= O_NONBLOCK;
    } else {
        flags &= ~O_NONBLOCK;
    }
    return fcntl(fd, F_SETFL, flags);
}

int siko_open_file(const char *path, int flags, int mode) {
#if defined(O_CLOEXEC)
    return open(path, flags | O_CLOEXEC, (mode_t)mode);
#else
    int fd = open(path, flags, (mode_t)mode);
    if (fd >= 0 && siko_set_cloexec(fd) < 0) {
        close(fd);
        return -1;
    }
    return fd;
#endif
}

int siko_socket_bind(int sockfd, const void *addr, int addrlen) {
    return bind(sockfd, (const struct sockaddr *)addr, (socklen_t)addrlen);
}

int siko_socket_connect(int sockfd, const void *addr, int addrlen) {
    return connect(sockfd, (const struct sockaddr *)addr, (socklen_t)addrlen);
}
