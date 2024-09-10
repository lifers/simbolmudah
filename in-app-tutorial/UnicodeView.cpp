module;
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
module TutorialDialog:UnicodeView;

namespace tut
{
    using namespace winrt::Microsoft::UI::Xaml;
    Controls::StackPanel UnicodeView(ResourceDictionary const&)
    {
        const Controls::TextBlock text{};
        text.Text(L"👋🌍🚀");

        const Controls::StackPanel panel;
        panel.Children().Append(text);
        return panel;
    }
}