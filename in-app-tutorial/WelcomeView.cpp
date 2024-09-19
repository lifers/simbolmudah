module;
#include <corecrt_memcpy_s.h>
module TutorialDialog:WelcomeView;

namespace tut
{
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;

    Controls::ScrollViewer WelcomeView(ResourceDictionary const& resCache)
    {
        const Controls::TextBlock title{};
        title.Text(L"Welcome to SimbolMudah!");
        title.TextWrapping(TextWrapping::Wrap);
        title.Style(resCache.Lookup(box_value(L"TitleTextBlockStyle")).as<Style>());

        const auto& secondaryBrush{ resCache.Lookup(box_value(L"TextFillColorSecondaryBrush")).as<Media::Brush>() };

        const Controls::TextBlock desc{};
        desc.Text(L"simbolmudah helps you enter many Unicode characters only with a few keystrokes.");
        desc.TextWrapping(TextWrapping::Wrap);
        desc.Foreground(secondaryBrush);

        const Controls::TextBlock instr{};
        instr.Text(L"Before we begin, find the Right Alt or AltGr key on your keyboard.");
        instr.TextWrapping(TextWrapping::Wrap);
        instr.Foreground(secondaryBrush);

        const Controls::TextBlock proceed{};
        proceed.Text(L"For this tutorial, swipe right to go to the next step, or left to go back.");
        proceed.TextWrapping(TextWrapping::Wrap);
        proceed.Foreground(secondaryBrush);

        const Controls::StackPanel panel{};
        panel.HorizontalAlignment(HorizontalAlignment::Center);
        panel.Padding(ThicknessHelper::FromUniformLength(32));
        panel.Spacing(16);
        panel.Children().ReplaceAll({ title, desc, instr, proceed });

        const Controls::ScrollViewer scroll{};
        scroll.Content(panel);
        return scroll;
    }
}