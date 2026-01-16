#include "set.hpp"
#include "dpi_stage.hpp"
#include "hidapi.h"

namespace SetCommand {
    void dpi_stage(hid_device* device, SetOptions const &opt) {
        auto packet = DPIStage::get_magic_packet(opt.dpi_stage);
        if (hid_send_feature_report(device, packet.data(), packet.size()) == -1) {
            std::cout << "Failed to send command." << std::endl;
        }
    }

    void setup_set_subcommand(CLI::App &app, hid_device* device) {
        auto opt = std::make_shared<SetOptions>();
        auto set_cmd = app.add_subcommand("set", "Set device parameters");

        set_cmd->add_option("-s,--dpi-stage,", opt->dpi_stage, "DPI stage to enable");

        set_cmd->callback([device, opt]() { dpi_stage(device, *opt); });
    }
}
