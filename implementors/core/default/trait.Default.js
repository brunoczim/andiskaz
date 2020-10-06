(function() {var implementors = {};
implementors["andiskaz"] = [{"text":"impl Default for DiacriticAtStart","synthetic":false,"types":[]},{"text":"impl Default for NotAGrapheme","synthetic":false,"types":[]},{"text":"impl Default for TermString","synthetic":false,"types":[]},{"text":"impl Default for TermGrapheme","synthetic":false,"types":[]},{"text":"impl Default for Coord2","synthetic":false,"types":[]},{"text":"impl Default for Id","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Default, U:&nbsp;Default&gt; Default for Seq&lt;T, U&gt;","synthetic":false,"types":[]},{"text":"impl&lt;F:&nbsp;Default&gt; Default for FromFn&lt;F&gt;","synthetic":false,"types":[]},{"text":"impl Default for Invert","synthetic":false,"types":[]},{"text":"impl&lt;F:&nbsp;Default, B:&nbsp;Default&gt; Default for PairWith&lt;F, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Transformer,<br>&nbsp;&nbsp;&nbsp;&nbsp;B: Transformer,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;F:&nbsp;Default, B:&nbsp;Default&gt; Default for SwapPairUsing&lt;F, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Transformer,<br>&nbsp;&nbsp;&nbsp;&nbsp;B: Transformer,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Default for AdaptFgToBg","synthetic":false,"types":[]},{"text":"impl Default for AdaptBgToFg","synthetic":false,"types":[]},{"text":"impl Default for ContrastFgWithBg","synthetic":false,"types":[]},{"text":"impl Default for ContrastBgWithFg","synthetic":false,"types":[]},{"text":"impl Default for Color2","synthetic":false,"types":[]},{"text":"impl&lt;P&gt; Default for Style&lt;P&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;P: PairTransformer + Default,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Default for ListenerFailed","synthetic":false,"types":[]},{"text":"impl Default for Tile","synthetic":false,"types":[]},{"text":"impl Default for Builder","synthetic":false,"types":[]}];
implementors["arc_swap"] = [{"text":"impl Default for Shard","synthetic":false,"types":[]},{"text":"impl Default for Global","synthetic":false,"types":[]},{"text":"impl Default for PrivateUnsharded","synthetic":false,"types":[]},{"text":"impl&lt;S:&nbsp;Default&gt; Default for PrivateSharded&lt;S&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;RefCnt + Default, S:&nbsp;LockStorage&gt; Default for ArcSwapAny&lt;T, S&gt;","synthetic":false,"types":[]}];
implementors["bytes"] = [{"text":"impl Default for Bytes","synthetic":false,"types":[]},{"text":"impl Default for BytesMut","synthetic":false,"types":[]}];
implementors["crossterm"] = [{"text":"impl Default for Attributes","synthetic":false,"types":[]},{"text":"impl Default for ContentStyle","synthetic":false,"types":[]}];
implementors["fnv"] = [{"text":"impl Default for FnvHasher","synthetic":false,"types":[]}];
implementors["lock_api"] = [{"text":"impl&lt;R:&nbsp;RawMutex, T:&nbsp;?Sized + Default&gt; Default for Mutex&lt;R, T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;RawMutex, G:&nbsp;GetThreadId, T:&nbsp;?Sized + Default&gt; Default for ReentrantMutex&lt;R, G, T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;RawRwLock, T:&nbsp;?Sized + Default&gt; Default for RwLock&lt;R, T&gt;","synthetic":false,"types":[]}];
implementors["parking_lot"] = [{"text":"impl Default for Condvar","synthetic":false,"types":[]},{"text":"impl Default for Once","synthetic":false,"types":[]}];
implementors["parking_lot_core"] = [{"text":"impl Default for UnparkResult","synthetic":false,"types":[]},{"text":"impl Default for SpinWait","synthetic":false,"types":[]}];
implementors["proc_macro2"] = [{"text":"impl Default for TokenStream","synthetic":false,"types":[]}];
implementors["slab"] = [{"text":"impl&lt;T&gt; Default for Slab&lt;T&gt;","synthetic":false,"types":[]}];
implementors["smallvec"] = [{"text":"impl&lt;A:&nbsp;Array&gt; Default for SmallVec&lt;A&gt;","synthetic":false,"types":[]}];
implementors["syn"] = [{"text":"impl Default for Underscore","synthetic":false,"types":[]},{"text":"impl Default for Abstract","synthetic":false,"types":[]},{"text":"impl Default for As","synthetic":false,"types":[]},{"text":"impl Default for Async","synthetic":false,"types":[]},{"text":"impl Default for Auto","synthetic":false,"types":[]},{"text":"impl Default for Await","synthetic":false,"types":[]},{"text":"impl Default for Become","synthetic":false,"types":[]},{"text":"impl Default for Box","synthetic":false,"types":[]},{"text":"impl Default for Break","synthetic":false,"types":[]},{"text":"impl Default for Const","synthetic":false,"types":[]},{"text":"impl Default for Continue","synthetic":false,"types":[]},{"text":"impl Default for Crate","synthetic":false,"types":[]},{"text":"impl Default for Default","synthetic":false,"types":[]},{"text":"impl Default for Do","synthetic":false,"types":[]},{"text":"impl Default for Dyn","synthetic":false,"types":[]},{"text":"impl Default for Else","synthetic":false,"types":[]},{"text":"impl Default for Enum","synthetic":false,"types":[]},{"text":"impl Default for Extern","synthetic":false,"types":[]},{"text":"impl Default for Final","synthetic":false,"types":[]},{"text":"impl Default for Fn","synthetic":false,"types":[]},{"text":"impl Default for For","synthetic":false,"types":[]},{"text":"impl Default for If","synthetic":false,"types":[]},{"text":"impl Default for Impl","synthetic":false,"types":[]},{"text":"impl Default for In","synthetic":false,"types":[]},{"text":"impl Default for Let","synthetic":false,"types":[]},{"text":"impl Default for Loop","synthetic":false,"types":[]},{"text":"impl Default for Macro","synthetic":false,"types":[]},{"text":"impl Default for Match","synthetic":false,"types":[]},{"text":"impl Default for Mod","synthetic":false,"types":[]},{"text":"impl Default for Move","synthetic":false,"types":[]},{"text":"impl Default for Mut","synthetic":false,"types":[]},{"text":"impl Default for Override","synthetic":false,"types":[]},{"text":"impl Default for Priv","synthetic":false,"types":[]},{"text":"impl Default for Pub","synthetic":false,"types":[]},{"text":"impl Default for Ref","synthetic":false,"types":[]},{"text":"impl Default for Return","synthetic":false,"types":[]},{"text":"impl Default for SelfType","synthetic":false,"types":[]},{"text":"impl Default for SelfValue","synthetic":false,"types":[]},{"text":"impl Default for Static","synthetic":false,"types":[]},{"text":"impl Default for Struct","synthetic":false,"types":[]},{"text":"impl Default for Super","synthetic":false,"types":[]},{"text":"impl Default for Trait","synthetic":false,"types":[]},{"text":"impl Default for Try","synthetic":false,"types":[]},{"text":"impl Default for Type","synthetic":false,"types":[]},{"text":"impl Default for Typeof","synthetic":false,"types":[]},{"text":"impl Default for Union","synthetic":false,"types":[]},{"text":"impl Default for Unsafe","synthetic":false,"types":[]},{"text":"impl Default for Unsized","synthetic":false,"types":[]},{"text":"impl Default for Use","synthetic":false,"types":[]},{"text":"impl Default for Virtual","synthetic":false,"types":[]},{"text":"impl Default for Where","synthetic":false,"types":[]},{"text":"impl Default for While","synthetic":false,"types":[]},{"text":"impl Default for Yield","synthetic":false,"types":[]},{"text":"impl Default for Add","synthetic":false,"types":[]},{"text":"impl Default for AddEq","synthetic":false,"types":[]},{"text":"impl Default for And","synthetic":false,"types":[]},{"text":"impl Default for AndAnd","synthetic":false,"types":[]},{"text":"impl Default for AndEq","synthetic":false,"types":[]},{"text":"impl Default for At","synthetic":false,"types":[]},{"text":"impl Default for Bang","synthetic":false,"types":[]},{"text":"impl Default for Caret","synthetic":false,"types":[]},{"text":"impl Default for CaretEq","synthetic":false,"types":[]},{"text":"impl Default for Colon","synthetic":false,"types":[]},{"text":"impl Default for Colon2","synthetic":false,"types":[]},{"text":"impl Default for Comma","synthetic":false,"types":[]},{"text":"impl Default for Div","synthetic":false,"types":[]},{"text":"impl Default for DivEq","synthetic":false,"types":[]},{"text":"impl Default for Dollar","synthetic":false,"types":[]},{"text":"impl Default for Dot","synthetic":false,"types":[]},{"text":"impl Default for Dot2","synthetic":false,"types":[]},{"text":"impl Default for Dot3","synthetic":false,"types":[]},{"text":"impl Default for DotDotEq","synthetic":false,"types":[]},{"text":"impl Default for Eq","synthetic":false,"types":[]},{"text":"impl Default for EqEq","synthetic":false,"types":[]},{"text":"impl Default for Ge","synthetic":false,"types":[]},{"text":"impl Default for Gt","synthetic":false,"types":[]},{"text":"impl Default for Le","synthetic":false,"types":[]},{"text":"impl Default for Lt","synthetic":false,"types":[]},{"text":"impl Default for MulEq","synthetic":false,"types":[]},{"text":"impl Default for Ne","synthetic":false,"types":[]},{"text":"impl Default for Or","synthetic":false,"types":[]},{"text":"impl Default for OrEq","synthetic":false,"types":[]},{"text":"impl Default for OrOr","synthetic":false,"types":[]},{"text":"impl Default for Pound","synthetic":false,"types":[]},{"text":"impl Default for Question","synthetic":false,"types":[]},{"text":"impl Default for RArrow","synthetic":false,"types":[]},{"text":"impl Default for LArrow","synthetic":false,"types":[]},{"text":"impl Default for Rem","synthetic":false,"types":[]},{"text":"impl Default for RemEq","synthetic":false,"types":[]},{"text":"impl Default for FatArrow","synthetic":false,"types":[]},{"text":"impl Default for Semi","synthetic":false,"types":[]},{"text":"impl Default for Shl","synthetic":false,"types":[]},{"text":"impl Default for ShlEq","synthetic":false,"types":[]},{"text":"impl Default for Shr","synthetic":false,"types":[]},{"text":"impl Default for ShrEq","synthetic":false,"types":[]},{"text":"impl Default for Star","synthetic":false,"types":[]},{"text":"impl Default for Sub","synthetic":false,"types":[]},{"text":"impl Default for SubEq","synthetic":false,"types":[]},{"text":"impl Default for Tilde","synthetic":false,"types":[]},{"text":"impl Default for Brace","synthetic":false,"types":[]},{"text":"impl Default for Bracket","synthetic":false,"types":[]},{"text":"impl Default for Paren","synthetic":false,"types":[]},{"text":"impl Default for Group","synthetic":false,"types":[]},{"text":"impl Default for Generics","synthetic":false,"types":[]},{"text":"impl Default for BoundLifetimes","synthetic":false,"types":[]},{"text":"impl Default for PathArguments","synthetic":false,"types":[]},{"text":"impl&lt;T, P&gt; Default for Punctuated&lt;T, P&gt;","synthetic":false,"types":[]}];
implementors["tokio"] = [{"text":"impl Default for DirBuilder","synthetic":false,"types":[]},{"text":"impl Default for OpenOptions","synthetic":false,"types":[]},{"text":"impl Default for Builder","synthetic":false,"types":[]},{"text":"impl&lt;T&gt; Default for Mutex&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Default,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Default for Notify","synthetic":false,"types":[]},{"text":"impl&lt;T&gt; Default for RwLock&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Default,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;T&gt; Default for DelayQueue&lt;T&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()