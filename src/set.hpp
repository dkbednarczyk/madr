#pragma once

#include <CLI/CLI.hpp>
#include "hidapi.h"

namespace SetCommand {
    struct SetOptions {
        uint8_t dpi_stage;
    };

    void setup(CLI::App &app, hid_device* device);
} // namespace SetCommand

