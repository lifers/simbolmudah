#pragma once

#include "PopupWindow.g.h"
#include "UnicodePopup.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct PopupWindow : PopupWindowT<PopupWindow>
    {
        explicit PopupWindow(
            const LibSimbolMudah::KeyboardTranslator& translator,
            const LibSimbolMudah::KeyboardHook& hook,
            const LibSimbolMudah::SequenceDefinition& definition);
        PopupWindow(const PopupWindow&) = delete;
        PopupWindow& operator=(const PopupWindow&) = delete;

    private:
        fire_and_forget OnKeyTranslated(const LibSimbolMudah::KeyboardTranslator& translator, const hstring& message) const;
        fire_and_forget OnStateChanged(const LibSimbolMudah::KeyboardHook& hook, uint8_t state) const;
        fire_and_forget DrawWindow() const;
        HWND GetWindowHandle() const;
        int32_t GetDpi() const;

        const LibSimbolMudah::KeyboardTranslator::OnKeyTranslated_revoker keyTranslatedToken;
        const LibSimbolMudah::KeyboardHook::OnStateChanged_revoker stateChangedToken;
        const Microsoft::UI::Xaml::Controls::Page defaultPage;
        const simbolmudah_ui::SequencePopup sequencePopup;
        const simbolmudah_ui::SearchPopup searchPopup;
        const simbolmudah_ui::UnicodePopup unicodePopup;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct PopupWindow : PopupWindowT<PopupWindow, implementation::PopupWindow>
    {
    };
}
