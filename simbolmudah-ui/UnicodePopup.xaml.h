#pragma once

#include "UnicodePopup.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct UnicodePopup : UnicodePopupT<UnicodePopup>
    {
        UnicodePopup(LibSimbolMudah::SequenceDefinition const& seqdef);
        Windows::Foundation::Collections::IObservableVector<hstring> HexCode() const;
        void ShowAnswer();
        void ResetState();

    private:
        const LibSimbolMudah::SequenceDefinition seqdef;
        Windows::Foundation::Collections::IObservableVector<hstring> hexCodes;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct UnicodePopup : UnicodePopupT<UnicodePopup, implementation::UnicodePopup>
    {
    };
}
