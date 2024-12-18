module;
#include <corecrt_memcpy_s.h>
export module TutorialDialog;

import pcm;

export void* winrt_make_in_app_tutorial_TutorialDialog();

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;
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

    struct TutorialDialog : TutorialDialogT<TutorialDialog>
    {
        TutorialDialog() = default;

        Controls::ContentDialog GetDialog(ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup);

    private:
        bool hookPopupState{ false };
    };
}

namespace winrt::in_app_tutorial::factory_implementation
{
    template <typename D, typename T, typename... I>
    struct __declspec(empty_bases) TutorialDialogT : implements<D, winrt::Windows::Foundation::IActivationFactory, I...>
    {
        using instance_type = in_app_tutorial::TutorialDialog;

        hstring GetRuntimeClassName() const
        {
            return L"in_app_tutorial.TutorialDialog";
        }
        auto ActivateInstance() const
        {
            return make<T>();
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