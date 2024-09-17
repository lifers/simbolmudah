module;
#include <corecrt_memcpy_s.h>
#include <crtdbg.h>
module Common:SequenceTemplate;

import pcm;

using namespace winrt;
using namespace Microsoft::UI::Xaml;

namespace
{
    ResourceDictionary GetResourceCache()
    {
        const auto& res{ Application::Current().Resources() };
        _ASSERTE(res.HasKey(box_value(L"AccentFillColorDefaultBrush")));
        _ASSERTE(res.HasKey(box_value(L"AccentControlElevationBorderBrush")));
        _ASSERTE(res.HasKey(box_value(L"TextOnAccentFillColorPrimaryBrush")));
        _ASSERTE(res.HasKey(box_value(L"BodyStrongTextBlockStyle")));

        return res;
    }
}

namespace com
{
    SequenceTemplate::SequenceTemplate() : resourceCache(GetResourceCache()) {}

    UIElement SequenceTemplate::GetElement(ElementFactoryGetArgs const& args)
    {
        return CreateElement(this->resourceCache, args.Data().as<hstring>());
    }

    void SequenceTemplate::RecycleElement(ElementFactoryRecycleArgs const&)
    {
        // Do not recycle elements
    }

    Controls::Border CreateElement(ResourceDictionary const& resCache, hstring const& name)
    {
        const auto& borderBackground{ resCache.Lookup(box_value(L"AccentFillColorDefaultBrush")).as<Media::Brush>() };
        const auto& borderBrush{ resCache.Lookup(box_value(L"AccentControlElevationBorderBrush")).as<Media::Brush>() };
        const auto& textForeground{ resCache.Lookup(box_value(L"TextOnAccentFillColorPrimaryBrush")).as<Media::Brush>() };
        const auto& textStyle{ resCache.Lookup(box_value(L"BodyStrongTextBlockStyle")).as<Style>() };

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
        border.Padding(ThicknessHelper::FromLengths(4, 0, 4, 0));
        border.MinWidth(32);
        border.Height(32);
        border.CornerRadius(CornerRadiusHelper::FromUniformRadius(4));
        border.Child(text);

        return border;
    }

    std::vector<Controls::Border> CreateElement(ResourceDictionary const& resCache, std::vector<hstring> const& seq)
    {
        std::vector<Controls::Border> elements;
        for (const auto& key : seq)
        {
            elements.emplace_back(CreateElement(resCache, key));
        }
        return elements;
    }
}
