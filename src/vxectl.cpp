#include <iostream>
#include <hidapi/hidapi.h>
#include <CLI/CLI.hpp>
#include "set.hpp"

constexpr unsigned short MADR_VID = 0x373b;
constexpr unsigned short MADR_WIRED_PID = 0x103f;
constexpr unsigned short MADR_WIRELESS_PID = 0x1040;

hid_device* find_mouse_device() {
    struct hid_device_info *devs, *cur_dev;
    devs = hid_enumerate(MADR_VID, 0x0);
    cur_dev = devs;
    hid_device *device = nullptr;

    while (cur_dev) {
        if ((cur_dev->product_id == MADR_WIRED_PID || cur_dev->product_id == MADR_WIRELESS_PID)
            && cur_dev->interface_number == 1) {
            device = hid_open_path(cur_dev->path);
            if (device) break;
        }

        cur_dev = cur_dev->next;
    }

    hid_free_enumeration(devs);
    return device;
}

int main(int argc, char ** argv) {
    if (hid_init() == -1) {
        std::cout << "Failed to initialize HIDAPI: " << hid_error(nullptr) << std::endl;
        return -1;
    }

    hid_device *device = find_mouse_device();

    if (device == nullptr) {
        std::cout << "No compatible device found on Interface 1." << std::endl;
        hid_exit();
        return -1;
    }

    CLI::App app{"vxectl - Control your VXE gaming mouse from the command line"};
    app.require_subcommand();

    SetCommand::setup(app, device);

    CLI11_PARSE(app, argc, argv);

    hid_close(device);
    hid_exit();
    return 0;
}