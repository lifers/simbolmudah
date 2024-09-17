module;
#include <corecrt_memcpy_s.h>
module Common:TextBlock;

namespace com
{
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;

    Controls::TextBlock SecondaryTextBlock(ResourceDictionary const& resCache, hstring const& content)
    {
        const Controls::TextBlock text{};
        text.Foreground(resCache.Lookup(box_value(L"TextFillColorSecondaryBrush")).as<Media::Brush>());
        text.TextWrapping(TextWrapping::Wrap);
        text.VerticalAlignment(VerticalAlignment::Center);
        text.Text(content);
        return text;
    }
}