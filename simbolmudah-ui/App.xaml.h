#pragma once

#include "App.xaml.g.h"
#include <optional>

namespace winrt::simbolmudah_ui::implementation
{
    struct App : AppT<App>
    {
        App();
        App(const App&) = delete;
        App& operator=(const App&) = delete;

        void OnLaunched(Microsoft::UI::Xaml::LaunchActivatedEventArgs const&);
        void OnHookToggle(bool isOn);
        bool hookState{ false };
        const LibSimbolMudah::SequenceDefinition sequenceDefinition;
        const LibSimbolMudah::KeyboardTranslator keyboardTranslator;
		const apartment_context main_thread;

    private:
        fire_and_forget BuildDefinition() const;
        void InitializeSettings();

        Microsoft::UI::Xaml::Window window{ nullptr };
        std::optional<simbolmudah_ui::BlankWindow> popup;
        std::optional<LibSimbolMudah::KeyboardHook> keyboardHook;
    };
}
