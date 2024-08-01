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

    private:
        void WindowClosed(IInspectable const&, Microsoft::UI::Xaml::WindowEventArgs const&);
        fire_and_forget BuildDefinition() const;
        void OnSettingsChanged(IInspectable const&, Microsoft::UI::Xaml::Data::PropertyChangedEventArgs const&);
        fire_and_forget OnOpenSettings(LibSimbolMudah::NotifyIcon const&, bool);
        fire_and_forget OnNotifyIconSetHook(LibSimbolMudah::NotifyIcon const&, bool status);

        const apartment_context main_thread;
        const simbolmudah_ui::AppManager appManager;
        const LibSimbolMudah::SequenceDefinition sequenceDefinition;
        const LibSimbolMudah::KeyboardTranslator keyboardTranslator;

        simbolmudah_ui::MainWindow mainWindow{ nullptr };
        LibSimbolMudah::KeyboardHook keyboardHook{ nullptr };
        simbolmudah_ui::PopupWindow popupWindow{ nullptr };
        LibSimbolMudah::NotifyIcon notifyIcon{ nullptr };

        const simbolmudah_ui::AppManager::PropertyChanged_revoker settingsChangedRevoker;
        event_token openSettingsToken;
        event_token notifyIconSetHookToken;
    };
}