module;
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
module TutorialDialog:NormalView;

namespace tut
{
    using namespace winrt::Microsoft::UI::Xaml;
    Controls::StackPanel NormalView(ResourceDictionary const&)
    {
        const Controls::TextBlock text{};
        text.Text(L"NormalView");

        const Controls::StackPanel panel;
        panel.Children().Append(text);
        return panel;
    }
}