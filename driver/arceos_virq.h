#ifndef _ARCEOS_VIRQ_H
#define _ARCEOS_VIRQ_H 1

#include <linux/sched.h>
#include <linux/interrupt.h>

irqreturn_t arceos_virq_handler(int irq, void *dev_id);

typedef struct task_struct* virq_handler_t;

int register_virq_handler(virq_handler_t handler);
int unregister_virq_handler(virq_handler_t handler);
void signal_virq_handlers(int virq);

#endif
