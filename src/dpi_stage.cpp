#include "dpi_stage.hpp"
#include <vector>

namespace DPIStage {
    std::vector<uint8_t> get_magic_packet(int dpi_stage) {
        return {
            0x08,
            0x07,
            0x00,
            0x00,
            0x00,
            0x06,
            0x01,
            0x54,
            0x04,
            0x51,
            static_cast<uint8_t>(dpi_stage),
            static_cast<uint8_t>(0x55 - dpi_stage),
            0x00,
            0x00,
            0x00,
            0x00,
            0x41,
        };
    }
}