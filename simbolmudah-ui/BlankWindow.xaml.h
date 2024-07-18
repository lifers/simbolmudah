#pragma once

#include "BlankWindow.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct BlankWindow : BlankWindowT<BlankWindow>
    {
        explicit BlankWindow(
            const LibSimbolMudah::KeyboardTranslator& translator,
            const LibSimbolMudah::KeyboardHook& hook,
            const LibSimbolMudah::SequenceDefinition& definition);
        BlankWindow(const BlankWindow&) = delete;
        BlankWindow& operator=(const BlankWindow&) = delete;

    private:
        fire_and_forget OnKeyTranslated(const LibSimbolMudah::KeyboardTranslator& translator, const hstring& message) const;
        fire_and_forget OnStateChanged(const LibSimbolMudah::KeyboardHook& hook, uint8_t state) const;

        const apartment_context main_thread;
        const LibSimbolMudah::KeyboardTranslator translator;
        const LibSimbolMudah::KeyboardTranslator::OnKeyTranslated_revoker keyTranslatedToken;
        const LibSimbolMudah::KeyboardHook hook;
        const LibSimbolMudah::KeyboardHook::OnStateChanged_revoker stateChangedToken;
        const Microsoft::UI::Xaml::Controls::Page defaultPage;
        const simbolmudah_ui::SequencePopup sequencePopup;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct BlankWindow : BlankWindowT<BlankWindow, implementation::BlankWindow>
    {
    };
}
