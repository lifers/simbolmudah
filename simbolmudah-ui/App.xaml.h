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
        fire_and_forget OpenWindow();
        void SwitchTutorialDialog(bool state, Microsoft::UI::Xaml::XamlRoot const& src);

    private:
        Windows::Foundation::IAsyncAction RebuildDefinition();
        void OnSettingsChanged(IInspectable const&, Microsoft::UI::Xaml::Data::PropertyChangedEventArgs const&);
        void SwitchPopupWindow(bool state);
        void SwitchNotifyIcon(bool state);
        void SwitchKeyboardHook(bool state);
        fire_and_forget OnOpenSettings(LibSimbolMudah::NotifyIcon const&, bool);
        fire_and_forget OnNotifyIconSetHook(LibSimbolMudah::NotifyIcon const&, bool status);
        fire_and_forget OnAppExit(LibSimbolMudah::NotifyIcon const&, bool);

        const Microsoft::UI::Dispatching::DispatcherQueue mainThread;
        const simbolmudah_ui::AppManager appManager;

        weak_ref<simbolmudah_ui::MainWindow> mainWindow;
        const LibSimbolMudah::SequenceDefinition sequenceDefinition;
        const LibSimbolMudah::KeyboardTranslator keyboardTranslator;
        LibSimbolMudah::KeyboardHook keyboardHook{ nullptr };
        simbolmudah_ui::PopupWindow popupWindow{ nullptr };
        LibSimbolMudah::NotifyIcon notifyIcon{ nullptr };

        const simbolmudah_ui::AppManager::PropertyChanged_revoker settingsChangedRevoker;
        event_token openSettingsToken;
        event_token notifyIconSetHookToken;
        event_token appExitToken;
        Microsoft::UI::Xaml::Controls::ContentDialog tutorialDialog{ nullptr };
        Microsoft::UI::Xaml::Controls::ContentDialog::Opened_revoker tutorialOpenedToken;
        Microsoft::UI::Xaml::Controls::ContentDialog::Closing_revoker tutorialClosingToken;
        Windows::Foundation::IAsyncAction buildProgress;

        const hstring notifyIconPath;
    };
}