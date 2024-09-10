module;
#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
module TutorialDialog:ClosingView;

namespace tut
{
    using namespace winrt::Microsoft::UI::Xaml;
    Controls::StackPanel ClosingView(ResourceDictionary const&)
    {
        const Controls::TextBlock text{};
        text.Text(L"Thank you for using the tutorial!");

        const Controls::StackPanel panel;
        panel.Children().Append(text);
        return panel;
    }
}