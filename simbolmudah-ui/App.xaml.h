#pragma once

#include "App.xaml.g.h"

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
        fire_and_forget OnOpenSettings(LibSimbolMudah::NotifyIcon const&, bool);

        simbolmudah_ui::MainWindow window{ nullptr };
        simbolmudah_ui::PopupWindow popup{ nullptr };
        LibSimbolMudah::NotifyIcon notifyIcon{ nullptr };
        event_token onSettingsOpenedToken;
        LibSimbolMudah::KeyboardHook keyboardHook{ nullptr };
    };
}
