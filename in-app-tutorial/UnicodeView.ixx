export module TutorialDialog:UnicodeView;

import pcm;

namespace tut
{
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;
    using namespace Windows::Foundation;
    export Controls::ScrollView UnicodeView(
        ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup, bool& state, Controls::FlipView const& parent);
}