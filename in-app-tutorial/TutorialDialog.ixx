module;
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
export module TutorialDialog;

import pcm;

export void* winrt_make_in_app_tutorial_TutorialDialog();

namespace winrt::in_app_tutorial::implementation
{
    template <typename D, typename... I>
    struct __declspec(empty_bases) TutorialDialog_base : implements<D, in_app_tutorial::TutorialDialog, I...>
    {
        using base_type = TutorialDialog_base;
        using class_type = in_app_tutorial::TutorialDialog;
        using implements_type = typename TutorialDialog_base::implements_type;
        using implements_type::implements_type;
        
        hstring GetRuntimeClassName() const
        {
            return L"in_app_tutorial.TutorialDialog";
        }
    };

    template <typename D, typename... I>
    using TutorialDialogT = TutorialDialog_base<D, I...>;

    using namespace Microsoft::UI::Xaml;
    using namespace Windows::Foundation;
    using namespace LibSimbolMudah;
    struct TutorialDialog : TutorialDialogT<TutorialDialog>
    {
        TutorialDialog() = delete;

        static void Initialize(ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup);
        static Controls::ContentDialog GetDialog();
    };
}

namespace winrt::in_app_tutorial::factory_implementation
{
    using namespace Microsoft::UI::Xaml;
    using namespace Windows::Foundation;
    using namespace LibSimbolMudah;
    template <typename D, typename T, typename... I>
    struct __declspec(empty_bases) TutorialDialogT : implements<D, IActivationFactory, in_app_tutorial::ITutorialDialogStatics, I...>
    {
        using instance_type = in_app_tutorial::TutorialDialog;

        hstring GetRuntimeClassName() const
        {
            return L"in_app_tutorial.TutorialDialog";
        }
        auto Initialize(ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup)
        {
            return T::Initialize(resCache, hookPopup);
        }
        auto GetDialog()
        {
            return T::GetDialog();
        }
        [[noreturn]] IInspectable ActivateInstance() const
        {
            throw hresult_not_implemented();
        }
    };

    struct TutorialDialog : TutorialDialogT<TutorialDialog, implementation::TutorialDialog>
    {
    };
}

#if __has_include("TutorialDialog.g.cpp")
#define WINRT_EXPORT export
#include "TutorialDialog.g.cpp"
#endif