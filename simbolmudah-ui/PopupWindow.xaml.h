#pragma once

#include "PopupWindow.g.h"

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
        int32_t GetDpi() const;

        const apartment_context main_thread;
        const LibSimbolMudah::KeyboardTranslator translator;
        const LibSimbolMudah::KeyboardTranslator::OnKeyTranslated_revoker keyTranslatedToken;
        const LibSimbolMudah::KeyboardHook hook;
        const LibSimbolMudah::KeyboardHook::OnStateChanged_revoker stateChangedToken;
        const Microsoft::UI::Xaml::Controls::Page defaultPage;
        const simbolmudah_ui::SequencePopup sequencePopup;
        const simbolmudah_ui::SearchPopup searchPopup;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct PopupWindow : PopupWindowT<PopupWindow, implementation::PopupWindow>
    {
    };
}
