module;
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
module TutorialDialog:NormalView;

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;

namespace tut
{
    Controls::StackPanel NormalView(
        ResourceDictionary const&, EventHandler<bool> const& hookPopup)
    {
        const Controls::TextBlock text{};
        text.Text(L"NormalView");

        const Controls::Button button{};
        button.Content(box_value(L"Click me"));
        button.Click([hookPopup](IInspectable const& src, auto&&) { hookPopup(src, true); });
        button.Unloaded([hookPopup](IInspectable const& src, auto&&) { hookPopup(src, false); });

        const Controls::StackPanel panel;
        panel.Children().ReplaceAll({ text, button });
        return panel;
    }
}