module;
#include <Windows.h>
#include <winrt/LibSimbolMudah.h>
export module NotifyIcon;

namespace simbolmudah_ui
{
    export struct NotifyIcon
    {
        explicit NotifyIcon(HWND hwnd);
        NotifyIcon(const NotifyIcon&) = delete;
        NotifyIcon& operator=(const NotifyIcon&) = delete;
        ~NotifyIcon();

        void SubscribeStateChanged(const winrt::LibSimbolMudah::KeyboardHook& hook);
        void UnsubscribeStateChanged(const winrt::LibSimbolMudah::KeyboardHook& hook);

    private:
        NOTIFYICONDATAW nid;
        winrt::hstring icon_path;
        winrt::event_token stateChangedToken;

        winrt::fire_and_forget Initialize();
        void OnStateChanged(const winrt::LibSimbolMudah::KeyboardHook& hook, uint8_t state);
    };
}