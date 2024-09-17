﻿module;
#include <corecrt_memcpy_s.h>
module TutorialDialog:SearchView;

import Common;

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;
namespace tut
{
    Controls::ScrollView SearchView(
        ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup, bool& state, Controls::FlipView const& parent)
    {
        const Controls::TextBlock title{};
        title.Text(L"Search the Name");
        title.TextWrapping(TextWrapping::Wrap);
        title.Style(resCache.Lookup(box_value(L"TitleTextBlockStyle")).as<Style>());

        const Controls::TextBlock desc{};
        desc.Text(L"If you forget the sequence, you can search the character by name.");
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
        switcher.Loading([&state](UIElement const& src, auto&&) {
            const auto srcSwitch{ src.as<Controls::ToggleSwitch>() };
            srcSwitch.IsOn(state);
        });
        parent.SelectionChanged([&state, switcher](auto&&, auto&&) {
            switcher.IsOn(state);
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

        const Controls::StackPanel seq1Panel{};
        seq1Panel.Orientation(Controls::Orientation::Horizontal);
        seq1Panel.VerticalAlignment(VerticalAlignment::Center);
        seq1Panel.Spacing(4);
        const auto children1{ seq1Panel.Children() };
        children1.Append(com::SecondaryTextBlock(resCache, L"Press"));
        for (const auto& seq : { L"AltGr", L"AltGr" })
        {
            children1.Append(com::CreateElement(resCache, seq));
        }
        children1.Append(com::SecondaryTextBlock(resCache, L"and type the name."));

        const Controls::StackPanel panel{};
        panel.HorizontalAlignment(HorizontalAlignment::Center);
        panel.Padding(ThicknessHelper::FromUniformLength(32));
        panel.Spacing(16);
        panel.Children().ReplaceAll({ title, desc, seq1Panel, switcherGrid });

        const Controls::ScrollView scroll{};
        scroll.Content(panel);
        return scroll;
    }
}