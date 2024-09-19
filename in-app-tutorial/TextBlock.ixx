export module Common:TextBlock;

import pcm;

namespace com
{
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;

    export Controls::TextBlock SecondaryTextBlock(ResourceDictionary const& resCache, hstring const& content);
}