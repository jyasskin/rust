#include "rust_internal.h"
#include "rust_port.h"

rust_port::rust_port(rust_task *task, size_t unit_sz) :
    maybe_proxy<rust_port>(this), task(task), unit_sz(unit_sz),
    writers(task->dom), chans(task->dom) {

    task->log(rust_log::MEM | rust_log::COMM,
              "new rust_port(task=0x%" PRIxPTR ", unit_sz=%d) -> port=0x%"
              PRIxPTR, (uintptr_t)task, unit_sz, (uintptr_t)this);

    // Allocate a remote channel, for remote channel data.
    remote_channel = new (task->dom) rust_chan(task, this);
}

rust_port::~rust_port() {
    task->log(rust_log::COMM | rust_log::MEM,
              "~rust_port 0x%" PRIxPTR, (uintptr_t) this);

    log_state();

    // Disassociate channels from this port.
    while (chans.is_empty() == false) {
        rust_chan *chan = chans.peek();
        chan->disassociate();
    }

    delete remote_channel;
}

bool rust_port::receive(void *dptr) {
    for (uint32_t i = 0; i < chans.length(); i++) {
        rust_chan *chan = chans[i];
        if (chan->buffer.is_empty() == false) {
            chan->buffer.dequeue(dptr);
            if (chan->buffer.is_empty() && chan->task->blocked()) {
                task->log(rust_log::COMM,
                          "chan: 0x%" PRIxPTR
                          " is flushing, wakeup task: 0x%" PRIxPTR,
                          chan, chan->task);
                chan->task->wakeup(this);
            }
            task->log(rust_log::COMM, "<=== read data ===");
            return true;
        }
    }
    return false;
}

void rust_port::log_state() {
    task->log(rust_log::COMM,
              "rust_port: 0x%" PRIxPTR ", associated channel(s): %d",
              this, chans.length());
    for (uint32_t i = 0; i < chans.length(); i++) {
        rust_chan *chan = chans[i];
        task->log(rust_log::COMM,
            "\tchan: 0x%" PRIxPTR ", data pending: %s, remote: %s",
            chan,
            !chan->buffer.is_empty() ? "yes" : "no",
            chan == remote_channel ? "yes" : "no");
    }
}

//
// Local Variables:
// mode: C++
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// compile-command: "make -k -C .. 2>&1 | sed -e 's/\\/x\\//x:\\//g'";
// End:
//
