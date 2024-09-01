#pragma once

#include "CustomSeqPage.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct CustomSeqPage : CustomSeqPageT<CustomSeqPage> {};
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct CustomSeqPage : CustomSeqPageT<CustomSeqPage, implementation::CustomSeqPage> {};
}
