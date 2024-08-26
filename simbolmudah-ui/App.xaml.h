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

    private:
        Windows::Foundation::IAsyncAction BuildDefinitionAndReset() const;
        void OnSettingsChanged(IInspectable const&, Microsoft::UI::Xaml::Data::PropertyChangedEventArgs const&);
        fire_and_forget OnNotifyIconPathInitialized(
            Windows::Foundation::IAsyncOperation<Windows::Storage::StorageFile> const& op,
            Windows::Foundation::AsyncStatus);
        void InitializeNotifyIcon();
        void InitializeKeyboardHook();
        fire_and_forget OnOpenSettings(LibSimbolMudah::NotifyIcon const&, bool);
        fire_and_forget OnNotifyIconSetHook(LibSimbolMudah::NotifyIcon const&, bool status);
        fire_and_forget OnAppExit(LibSimbolMudah::NotifyIcon const&, bool);

        const Microsoft::UI::Dispatching::DispatcherQueue main_thread;
        const Microsoft::UI::Dispatching::DispatcherQueueController keyboardThread;
        const simbolmudah_ui::AppManager appManager;
        const LibSimbolMudah::SequenceDefinition sequenceDefinition;
        const LibSimbolMudah::KeyboardTranslator keyboardTranslator;

        weak_ref<simbolmudah_ui::MainWindow> mainWindow;
        LibSimbolMudah::KeyboardHook keyboardHook{ nullptr };
        simbolmudah_ui::PopupWindow popupWindow{ nullptr };
        LibSimbolMudah::NotifyIcon notifyIcon{ nullptr };

        const simbolmudah_ui::AppManager::PropertyChanged_revoker settingsChangedRevoker;
        event_token openSettingsToken;
        event_token notifyIconSetHookToken;
        event_token appExitToken;
        Windows::Foundation::IAsyncAction buildProgress;

        hstring notifyIconPath;
        bool delayNotifyIcon{ false };
    };
}