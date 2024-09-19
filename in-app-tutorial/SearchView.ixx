export module TutorialDialog:SearchView;

import pcm;

namespace tut
{
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;
    using namespace Windows::Foundation;
    export Controls::ScrollView SearchView(
        ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup, bool& state, Controls::FlipView const& parent);
}