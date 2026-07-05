#ifndef SIKO_RUNTIME_H
#define SIKO_RUNTIME_H

#include <stdint.h>
#include <stddef.h>

int siko_pthread_join(uint64_t thread, void **retval);

int siko_pipe_create(int pipefd[2]);
int siko_fd_set_nonblocking(int fd, int nonblocking);
int siko_open_file(const char *path, int flags, int mode);

int siko_socket_bind(int sockfd, const void *addr, int addrlen);
int siko_socket_connect(int sockfd, const void *addr, int addrlen);

#endif
