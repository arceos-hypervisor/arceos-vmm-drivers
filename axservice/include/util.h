#ifndef __UTIL_H__
#define __UTIL_H__

#define INFO(args...)                                                          \
    do {                                                                       \
        pr_err("[AX INFO] " args);                                                \
    } while (0)

#define WARNING(args...)                                                       \
    do {                                                                       \
        pr_err("[AX WARNING] " args);                                             \
    } while (0)

#define ERROR(args...)                                                         \
    do {                                                                       \
        pr_err("[AX ERROR] " args);                                               \
    } while (0)

#endif