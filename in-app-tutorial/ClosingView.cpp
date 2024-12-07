module;
#include <corecrt_memcpy_s.h>
module TutorialDialog:ClosingView;

import Common;

namespace tut
{
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;
    using namespace Windows::Foundation;
    Controls::ScrollView ClosingView(ResourceDictionary const& resCache) noexcept
    {
        const Controls::TextBlock goodbye{};
        goodbye.Text(L"You're set!");
        goodbye.TextWrapping(TextWrapping::Wrap);
        goodbye.Style(resCache.Lookup(box_value(L"TitleTextBlockStyle")).as<Style>());

        const auto instr{ com::SecondaryTextBlock(resCache, L"Go to Settings when you're ready to enable simbolmudah.\
 If you close this window, simbolmudah will run in the background. You can access it from the tray icon.") };

        const Documents::Run issuesText{};
        issuesText.Text(L"our GitHub issues.");
        const Documents::Hyperlink issues{};
        issues.NavigateUri(Uri(L"https://github.com/lifers/simbolmudah/issues"));
        issues.Inlines().Append(issuesText);

        const auto feedback{ com::SecondaryTextBlock(resCache, L"Please provide feedback to help us improve simbolmudah.\
 Send your feedback to ") };
        feedback.Inlines().Append(issues);

        const Controls::StackPanel panel{};
        panel.HorizontalAlignment(HorizontalAlignment::Center);
        panel.Padding(ThicknessHelper::FromUniformLength(32));
        panel.Spacing(16);
        panel.Children().ReplaceAll({ goodbye, instr, feedback });

        const Controls::ScrollView scroll{};
        scroll.Content(panel);
        return scroll;
    }
}