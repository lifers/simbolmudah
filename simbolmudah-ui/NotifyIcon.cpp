module;
#include <Windows.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Storage.h>
#include <winrt/LibSimbolMudah.h>
module NotifyIcon;

namespace simbolmudah_ui
{
    using namespace winrt;
    using namespace LibSimbolMudah;

    NotifyIcon::NotifyIcon(HWND hwnd) :
        nid{ .cbSize = sizeof(NOTIFYICONDATAW), .hWnd = hwnd, .uID = 0,
             .uFlags = NIF_ICON | NIF_TIP, .szTip = L"simbolmudah" }
    {
        this->Initialize();
    }

    NotifyIcon::~NotifyIcon()
    {
        ::Shell_NotifyIconW(NIM_DELETE, &this->nid);
    }

    void NotifyIcon::SubscribeStateChanged(const KeyboardHook& hook)
    {
        this->stateChangedToken = hook.OnStateChanged({ this, &NotifyIcon::OnStateChanged });
    }

    void NotifyIcon::UnsubscribeStateChanged(const KeyboardHook& hook)
    {
        hook.OnStateChanged(this->stateChangedToken);
    }

    fire_and_forget NotifyIcon::Initialize()
    {
        using namespace Windows;
        using namespace Foundation;
        using namespace Storage;

        const auto context{ apartment_context() };
        co_await resume_background();

        this->icon_path = StorageFile::GetFileFromApplicationUriAsync(
            Uri(L"ms-appx:///Images/favicon.ico")).get().Path();

        co_await context;
        this->nid.hIcon = ::LoadIconW(nullptr, IDI_WARNING);
        check_bool(::Shell_NotifyIconW(NIM_ADD, &this->nid));
    }

    void NotifyIcon::OnStateChanged(const KeyboardHook& hook, uint8_t state)
    {
        switch (state)
        {
        case 0: // Idle
            this->nid.hIcon = ::LoadIconW(nullptr, IDI_APPLICATION);
            check_bool(::Shell_NotifyIconW(NIM_MODIFY, &this->nid));
            return;
        case 2: // ComposeKeyupFirst
            this->nid.hIcon = ::LoadIconW(nullptr, IDI_ASTERISK);
            check_bool(::Shell_NotifyIconW(NIM_MODIFY, &this->nid));
            return;
        case 4: // SequenceMode
            this->nid.hIcon = ::LoadIconW(nullptr, IDI_QUESTION);
            check_bool(::Shell_NotifyIconW(NIM_MODIFY, &this->nid));
            return;
        case 5: // SearchMode
            this->nid.hIcon = ::LoadIconW(nullptr, IDI_WINLOGO);
            check_bool(::Shell_NotifyIconW(NIM_MODIFY, &this->nid));
            return;
        }
    }
}