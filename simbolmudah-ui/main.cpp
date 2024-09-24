#include "pch.hpp"
#include <wil/resource.h>
#include <winrt/Microsoft.Windows.AppLifecycle.h>
#include "App.xaml.h"


using namespace winrt;
using namespace winrt::Microsoft::UI::Xaml;
using namespace winrt::Microsoft::Windows::AppLifecycle;
using namespace simbolmudah_ui::implementation;

namespace
{
    void OnActivated(IInspectable const&, AppActivationArguments const&)
    {
        Application::Current().as<App>()->OpenWindow();
    }

    fire_and_forget Redirect(AppInstance const& keyInstance, AppActivationArguments const& args, wil::unique_event const& redirectHandle)
    {
        const auto ensure_signaled{ wil::SetEvent_scope_exit(redirectHandle.get()) };
        co_await keyInstance.RedirectActivationToAsync(args);
    }
}

int WINAPI wWinMain(_In_ HINSTANCE, _In_opt_ HINSTANCE, _In_ LPWSTR, _In_ int)
{
    init_apartment(apartment_type::single_threaded);

    if (const auto& keyInstance{ AppInstance::FindOrRegisterForKey(L"simbolmudah") }; keyInstance.IsCurrent())
    {
        keyInstance.Activated({ OnActivated });
        Application::Start([](auto&&) { make<App>(); });
    }
    else
    {
        wil::unique_event redirectHandle;
        redirectHandle.create();
        Redirect(keyInstance, AppInstance::GetCurrent().GetActivatedEventArgs(), redirectHandle);
        DWORD handleIndex{};
        const auto handle{ redirectHandle.get() };
        check_hresult(CoWaitForMultipleObjects(CWMO_DEFAULT, INFINITE, 1, &handle, &handleIndex));
    }

    return 0;
}