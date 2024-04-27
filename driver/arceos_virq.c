#include "arceos_virq.h"

#include "arceos_hypercall.h"
#include "definitions.h"

#include <linux/list.h>
#include <linux/sched/signal.h>
#include <linux/slab.h>

struct virq_handler_node {
    struct list_head entry;
    virq_handler_t handler;
};

static LIST_HEAD(virq_handlers);

irqreturn_t arceos_virq_handler(int irq, void *dev_id) {
    signal_virq_handlers(irq);

    return IRQ_HANDLED;
}

int register_virq_handler(virq_handler_t handler) {
    struct virq_handler_node *node;

    // check if it's already registered
    list_for_each_entry(node, &virq_handlers, entry) {
        if (node->handler == handler) {
            return -EEXIST;
        }
    }

    // add it to the list
    node = kzalloc(sizeof(struct virq_handler_node), GFP_KERNEL);
    INIT_LIST_HEAD(&node->entry);
    node->handler = handler;
    list_add_tail(&node->entry, &virq_handlers);

    return 0;
}

int unregister_virq_handler(virq_handler_t handler) {
    struct virq_handler_node *node;

    // remove it from the list
    list_for_each_entry(node, &virq_handlers, entry) {
        if (node->handler == handler) {
            list_del(&node->entry);
            kfree(node);
            return 0;
        }
    }

	return -EINVAL;
}

void signal_virq_handlers(int virq) {
    struct virq_handler_node *node;
    struct kernel_siginfo info;
    int err;

    memset(&info, 0, sizeof(struct kernel_siginfo));
    info.si_signo = ARCEOS_VIRQ_SIG_NUM;
    info.si_code = SI_QUEUE;
    info.si_int = virq;

    // send signals
    list_for_each_entry(node, &virq_handlers, entry) {
        err = send_sig_info(ARCEOS_VIRQ_SIG_NUM, &info, node->handler);
    }
}
