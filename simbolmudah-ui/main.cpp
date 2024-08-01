#include "pch.hpp"
#include <wil/resource.h>
#include <winrt/Microsoft.Windows.AppLifecycle.h>
#include "App.xaml.h"


using namespace winrt;

namespace
{
    using namespace wil;
    using namespace winrt::Microsoft::Windows::AppLifecycle;

    fire_and_forget Redirect(AppInstance keyInstance, AppActivationArguments args, unique_event& redirectHandle)
    {
        const auto ensure_signaled{ SetEvent_scope_exit(redirectHandle.get()) };
        co_await keyInstance.RedirectActivationToAsync(args);
    }
}

int WINAPI wWinMain(_In_ HINSTANCE, _In_opt_ HINSTANCE, _In_ LPWSTR, _In_ int)
{
    using namespace winrt::Microsoft::UI::Xaml;
    using namespace winrt::Microsoft::Windows::AppLifecycle;
    using namespace Windows::Foundation;

    init_apartment(apartment_type::single_threaded);

    if (const auto keyInstance{ AppInstance::FindOrRegisterForKey(L"simbolmudah") };
        keyInstance.IsCurrent())
    {
        Application::Start([](auto&&) { make<simbolmudah_ui::implementation::App>(); });
    }
    else
    {
        wil::unique_event redirectHandle;
        redirectHandle.create();
        Redirect(keyInstance, AppInstance::GetCurrent().GetActivatedEventArgs(), redirectHandle);
        DWORD handleIndex{};
        check_hresult(CoWaitForMultipleObjects(CWMO_DEFAULT, INFINITE, 1, redirectHandle.addressof(), &handleIndex));
    }

    return 0;
}