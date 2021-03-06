#include <cstdlib>
#include <iostream>
#include <string>

#include <clara/clara.hpp>
#include <fmt/format.h>
#include <zmq.hpp>

#include <oneseismic/azure.hpp>
#include <oneseismic/transfer.hpp>
#include <oneseismic/tasks.hpp>

namespace one {

class az_manifest : public az {
public:
    using az::az;

    std::string url(
            const one::batch& batch,
            const std::string&) const override {

        return fmt::format(
            "{}/{}/manifest.json",
            batch.storage_endpoint,
            batch.guid
        );

    }

};

}

int main(int argc, char** argv) {
    std::string source_address;
    std::string sink_address = "tcp://*:68142";
    std::string control_address;
    std::string fail_address;
    bool help = false;
    int ntransfers = 4;
    int task_size = 10;

    auto cli
        = clara::Help(help)
        | clara::Opt(sink_address, "sink")
            ["--sink"]
            (fmt::format("Sink address, default = {}", sink_address))
        | clara::Opt(source_address, "source")
            ["--source"]
            (fmt::format("source address"))
        | clara::Opt(control_address, "control")
            ["--control"]
            (fmt::format("control address, currently unused"))
        | clara::Opt(fail_address, "fail")
            ["--fail"]
            (fmt::format("failure address"))
        | clara::Opt(ntransfers, "transfers")
            ["-j"]["--transfers"]
            (fmt::format("Concurrent blob connections, default = {}", ntransfers))
        | clara::Opt(task_size, "task size")
            ["-t"]["--task-size"]
            (fmt::format("Max task size (# of fragments), default = {}", task_size))
    ;

    auto result = cli.parse(clara::Args(argc, argv));

    if (!result) {
        fmt::print(stderr, "{}\n", result.errorMessage());
        std::exit(EXIT_FAILURE);
    }

    if (help) {
        std::cout << cli << "\n";
        std::exit(EXIT_SUCCESS);
    }

    zmq::context_t ctx;
    zmq::socket_t source(ctx, ZMQ_PULL);
    zmq::socket_t sink(ctx, ZMQ_PUSH);
    zmq::socket_t control(ctx, ZMQ_SUB);
    zmq::socket_t fail(ctx, ZMQ_PUSH);
    control.setsockopt(ZMQ_SUBSCRIBE, "ctrl:kill", 0);

    try {
        source.connect(source_address);
    } catch (...) {
        std::cerr << "Invalid source address\n";
        std::exit(EXIT_FAILURE);
    }
    try {
        sink.bind(sink_address);
    } catch (...) {
        std::cerr << "Invalid sink address\n";
        std::exit(EXIT_FAILURE);
    }
    try {
        fail.connect(fail_address);
    } catch (...) {
        std::cerr << "Invalid failure address\n";
        std::exit(EXIT_FAILURE);
    }

    one::az_manifest az("");
    one::transfer xfer(ntransfers, az);
    one::manifest_task task;
    try {
        task.max_task_size(task_size);
    } catch (const std::exception& e) {
        std::cerr << e.what() << "\n";
        std::exit(EXIT_FAILURE);
    }

    zmq::pollitem_t items[] = {
        { static_cast< void* >(source),  0, ZMQ_POLLIN, 0 },
        { static_cast< void* >(control), 0, ZMQ_POLLIN, 0 },
    };

    while (true) {
        zmq::poll(items, 2, -1);

        if (items[0].revents & ZMQ_POLLIN) {
            task.run(xfer, source, sink, fail);
        }

        if (items[1].revents & ZMQ_POLLIN) {
            break;
        }
    }
}
