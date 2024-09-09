#pragma once
#include <winrt/Microsoft.UI.Xaml.Controls.h>
#include <winrt/LibSimbolMudah.h>

namespace winrt::simbolmudah_ui
{
    struct UnicodePopup
    {
        explicit UnicodePopup(winrt::LibSimbolMudah::SequenceDefinition const& seqdef);
        UnicodePopup(const UnicodePopup&) = delete;
        UnicodePopup& operator=(const UnicodePopup&) = delete;
        UnicodePopup(UnicodePopup&&) = delete;

        void ShowAnswer() const;
        void ResetState() const;

    private:
        const winrt::LibSimbolMudah::SequenceDefinition seqdef;
        const winrt::Microsoft::UI::Xaml::Controls::TextBlock resultText;
        const winrt::Microsoft::UI::Xaml::Controls::TextBlock descText;

    public:
        const winrt::Windows::Foundation::Collections::IObservableVector<winrt::hstring> hexCodes;
        const winrt::Microsoft::UI::Xaml::Controls::Page innerPage;
    };
}