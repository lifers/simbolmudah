#include "pch.hpp"
#include "SequenceTemplate.h"

using namespace winrt;
using namespace Microsoft::UI::Xaml;


namespace
{
    ResourceDictionary GetResourceCache()
    {
        const auto& res{ Application::Current().Resources() };
        WINRT_ASSERT(res.HasKey(box_value(L"AccentFillColorDefaultBrush")));
        WINRT_ASSERT(res.HasKey(box_value(L"AccentControlElevationBorderBrush")));
        WINRT_ASSERT(res.HasKey(box_value(L"TextOnAccentFillColorPrimaryBrush")));
        WINRT_ASSERT(res.HasKey(box_value(L"BodyStrongTextBlockStyle")));

        return res;
    }
}

SequenceTemplate::SequenceTemplate() : resourceCache(GetResourceCache()) {}

UIElement SequenceTemplate::GetElement(ElementFactoryGetArgs const& args)
{
    return this->CreateElement(args.Data().as<hstring>());
}

void SequenceTemplate::RecycleElement(ElementFactoryRecycleArgs const&)
{
    // Do not recycle elements
}

Controls::Border SequenceTemplate::CreateElement(hstring const& name)
{
    const auto& borderBackground{ this->resourceCache.Lookup(box_value(L"AccentFillColorDefaultBrush")).as<Media::Brush>() };
    const auto& borderBrush{ this->resourceCache.Lookup(box_value(L"AccentControlElevationBorderBrush")).as<Media::Brush>() };
    const auto& textForeground{ this->resourceCache.Lookup(box_value(L"TextOnAccentFillColorPrimaryBrush")).as<Media::Brush>() };
    const auto& textStyle{ this->resourceCache.Lookup(box_value(L"BodyStrongTextBlockStyle")).as<Style>() };

    const Controls::TextBlock text{};
    text.Foreground(textForeground);
    text.Style(textStyle);
    text.FontSize(16);
    text.HorizontalAlignment(HorizontalAlignment::Center);
    text.VerticalAlignment(VerticalAlignment::Center);
    text.Text(name);

    const Controls::Border border{};
    border.Background(borderBackground);
    border.BorderBrush(borderBrush);
    border.BorderThickness(ThicknessHelper::FromUniformLength(1));
    border.Width(32);
    border.Height(32);
    border.CornerRadius(CornerRadiusHelper::FromUniformRadius(4));
    border.Child(text);

    return border;
}