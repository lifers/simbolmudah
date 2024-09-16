module;
#include <corecrt_memcpy_s.h>
module TutorialDialog:NormalView;

import Common;

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;

namespace
{
    Controls::StackPanel CreateSequence(ResourceDictionary const& resCache, std::vector<hstring> seq, hstring const& ans)
    {
        Controls::TextBlock text1{};
        text1.Text(L"Press");
        text1.VerticalAlignment(VerticalAlignment::Center);
        text1.Style(resCache.Lookup(box_value(L"BodyStrongTextBlockStyle")).as<Style>());

        Controls::TextBlock text2{};
        text2.Text(L"to compose");
        text2.VerticalAlignment(VerticalAlignment::Center);
        text2.Style(resCache.Lookup(box_value(L"BodyStrongTextBlockStyle")).as<Style>());

        Controls::StackPanel panel{};
        panel.Orientation(Controls::Orientation::Horizontal);
        panel.VerticalAlignment(VerticalAlignment::Center);
        panel.Spacing(4);
        panel.Children().Append(text1);
        for (const auto& key : seq)
        {
            panel.Children().Append(com::CreateElement(resCache, key));
        }
        panel.Children().Append(text2);
        panel.Children().Append(com::CreateElement(resCache, ans));
        return panel;
    }
}

namespace tut
{
    Controls::ScrollView NormalView(
        ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup, bool& state)
    {
        const Controls::TextBlock title{};
        title.Text(L"Composing a Character");
        title.TextWrapping(TextWrapping::Wrap);
        title.Style(resCache.Lookup(box_value(L"TitleTextBlockStyle")).as<Style>());

        const Controls::TextBlock desc{};
        desc.Text(L"To compose a character, you need to press a certain sequence of keys without\
 holding any of them. Try out these examples!");
        desc.TextWrapping(TextWrapping::Wrap);
        desc.Foreground(resCache.Lookup(box_value(L"TextFillColorSecondaryBrush")).as<Media::Brush>());

        const Controls::StackPanel sequence{};
        sequence.Orientation(Controls::Orientation::Horizontal);
        sequence.VerticalAlignment(VerticalAlignment::Center);

        const Controls::ToggleSwitch switcher{};
        switcher.Header(box_value(L"Try it out!"));
        switcher.IsOn(state);
        switcher.Toggled([&state, hookPopup](IInspectable const& src, auto&&) {
            const auto srcSwitch{ src.as<Controls::ToggleSwitch>() };
            state = srcSwitch.IsOn();
            hookPopup(src, state);
        });

        const Controls::TextBox textBox{};
        textBox.PlaceholderText(L"Type here...");

        const Controls::ColumnDefinition col0{};
        col0.Width(GridLengthHelper::FromPixels(96));
        const Controls::ColumnDefinition col1{};

        const Controls::Grid switcherGrid{};
        switcherGrid.ColumnDefinitions().ReplaceAll({ col0, col1 });
        switcherGrid.Children().ReplaceAll({ switcher, textBox });
        switcherGrid.SetColumn(switcher, 0);
        switcherGrid.SetColumn(textBox, 1);

        const auto seq1{ CreateSequence(resCache, { L"AltGr", L"e", L"'" }, L"é") };
        const auto seq2{ CreateSequence(resCache, { L"AltGr", L"/", L"=" }, L"≠") };

        const Controls::StackPanel panel{};
        panel.HorizontalAlignment(HorizontalAlignment::Center);
        panel.Padding(ThicknessHelper::FromUniformLength(32));
        panel.Spacing(16);
        panel.Children().ReplaceAll({ title, desc, seq1, seq2, switcherGrid });

        const Controls::ScrollView scroll{};
        scroll.Content(panel);
        return scroll;
    }
}