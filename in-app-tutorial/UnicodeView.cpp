module;
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
module TutorialDialog:UnicodeView;

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;
namespace tut
{
    Controls::StackPanel UnicodeView(
        ResourceDictionary const&, EventHandler<bool> const&)
    {
        const Controls::TextBlock text{};
        text.Text(L"👋🌍🚀");

        const Controls::StackPanel panel;
        panel.Children().Append(text);
        return panel;
    }
}