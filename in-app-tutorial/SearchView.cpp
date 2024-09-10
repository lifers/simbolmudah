module;
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
module TutorialDialog:SearchView;

namespace tut
{
    using namespace winrt::Microsoft::UI::Xaml;
    Controls::StackPanel SearchView(ResourceDictionary const&)
    {
        const Controls::TextBlock text{};
        text.Text(L"SearchView");

        const Controls::StackPanel panel;
        panel.Children().Append(text);
        return panel;
    }
}