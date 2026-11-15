# thebluegecko reviewing our 1v1 against u6bkep, spoken, 2026-11-14

Data, not claims. A voice conversation the user recorded on 2026-11-14 from 22:43 (20:54 long), thebluegecko watching
the replay of the Comet Catcher 1v1 our seat lost to u6bkep that evening (`run/matches/1794707454-human`, 20:50 to
21:14, our seat Armada from G1, conceded at 21:36 of the game) with the user, u6bkep and thefiregecko in the channel.
The user's request: "we just got a review from thebluegecko on the most recent 1v1 vs u6bkep ... Review it, then come
back with the major points for us to update the brief with."

**The files and the speakers.** Mumble wrote one file a speaker (`~/Mumble-2026-11-14-22-43-18-10.0.0.2-<name>.wav`,
kept outside the repository): `thebluegecko(Linux)`, `u6bkep`, `thefiregecko` and `Recorder`. So the speaker of every
line below is the channel it was heard on, not a reading of the text. `Recorder` is taken to be the user's own
microphone (it speaks of "the bot", asks the questions and stops the recording); that one attribution is ours.

**How it was transcribed.** Locally, the same night, with faster-whisper large-v3 on the CPU (int8, beam 5, a BAR
vocabulary as the initial prompt, the voice filter on; nothing left the machine), each channel by itself;
thebluegecko's channel in four pieces cut at quiet moments. Every line is the model's words in its order and with
its punctuation, placed by the words' own clocks (recording time, not game time): a speaker's line ends where he
pauses for 2.5 s or where another speaker's words begin. The model's segments are not used for the lines, because one
segment runs up to half a minute (on a sparse channel, minutes) and would put an answer before its question. On the
three sparse channels the model wrote lower case without punctuation and dropped words, and their lines are the less
reliable ones. A second pass over
thebluegecko's channel with other settings (beam 2, no vocabulary, conditioned on its own previous text) agreed with
this one on every number and, in substance, on every sentence it heard (its differences are in the next paragraph); it
looped over 2:29-2:46, 8:42-8:58 and 15:19-15:21, so "u6bkep is only at 2.5, although it's evening up here quickly",
"Whenever you've got E excess, that means you've probably" and "the nearly 2k metal laying on the ground" rest on this
pass alone.

**Likely mis-hearings.** "mech"/"mechs" is "mex"/"mexes" throughout; "U6B cap" and "U6" are u6bkep; "mousers" are
Mausers (armmart); "Prophet" is right (armseer, the tier-2 radar vehicle); "Go ban" at 18:44 is probably "Go Ben" (Ben
is u6bkep); "the anti-valley pickings of that metal" at 8:59 was "the anti-value picking with that metal" in the second
pass and is unclear in both; "Three minutes to five seconds" at 2:07 is as heard by both passes (he is answering "what
timestamp are you at"); "V2 mechs" and "D2 mechs" at 18:47 and 19:08 are "T2 mexes"; "7,000 E each" for the advanced
extractor is as he said it (the unit file: 7,700). The second pass heard "comm vehicle" for "con vehicle" at 0:09 and
0:54 (the constructor is meant: the vocabulary pass is the better one there), "What can I add here?" for "Can I add
here" at 11:30 (so the Janus line is his own, not an interjection), "a big, big economic advantage" at 6:53 and "it'll
just go straight for bulls" at 17:24. Words doubled where two segments meet ("value you", "many any",
"advantage advantage", "commander. commander.") are the model's.

**The colours.** Blue is u6bkep and red is our seat: "our bot is at 3K excess E" and "red needs to do something
desperate" are the same side, and the tier-2 plant and the Mausers he calls blue's are u6bkep's.

**Where the recording is in the game.** He names three clocks himself: about 3:05 at 2:07 of the recording, 4:16 at
3:07, 7:30 at 6:31. The rest is placed from our record of the game, as a reading: "14k to 11k" metal produced at 8:59
is about 9:30 to 10:00 of the game (our income summed); "one of the two labs" at 9:51 follows our second vehicle plant,
begun 9:32; the converters at 12:13 are the four our commander began from 12:32 to 13:02; the fight he describes from
14:05 to 15:21 is the one at our base from 15:15 to 16:41 (2,650 metal of ours lost for 705); his tier-2 plant at 16:24
was first in our sight in the game's eighteenth minute; the end at 20:16 is the concession at 21:36.

[00:00] the user: i will make this easier myself all right uh so first thing was we need three solars before the uh vehicle plant all
[00:09] thebluegecko: Right, in order to get, that lets you rush out that con vehicle without stalling E.
[00:16] the user: right okay
[00:17] thebluegecko: It is very common on Comet Catcher to walk with your com to the closest mex when you run out of metal or when you're going to run out of metal. So if you spawn on a 2, build up to 4 solars, and then at that point you're out of metal. So walk to the nearest mex and then walk back with the com. If you spawn on a spot that lets you walk to a 2 mechs, that's better. But I know that may not be an option for our Claude here.
[00:36] u6bkep: That Hulk is going to go down soon. I
[00:54] thebluegecko: The first con vehicle should continue the way it was, rather than running all the way back. Try to set up a good construction path for each vehicle to minimize overlap walk spacing.
[01:16] thebluegecko: At 9 metal per second, you can get more military value you out of rovers than you can with blitzes. The main reason that ARM is currently favored on this map, and strongly favored among highly skilled human players, is rovers are stronger than rascals.
[01:43] thebluegecko: And right at the start, you can't actually afford to power out very many any blitzes because you metal stall. Right now, I see a fifth and sixth solar being built simultaneously versus a solid metal stall. Only one of these solar should have been built.
[02:04] the user: what timestamp are you at i
[02:07] thebluegecko: Three minutes to five seconds.
[02:12] the user: will get there okay
[02:14] thebluegecko: If there was more metal, this might be needed. One of the important things to do in the beginning is not waste a bunch of E. If we look at the E excess, both players here are high, but our bot is at 3K excess E, whereas U6B cap is only at 2.5, although it's evening up here quickly. Whenever
[02:44] thebluegecko: you've got E excess, that means you've probably built too many solars. If you're going to leave the comm in base, you can always keep your E at about the middle slider area because you always have that chance has to immediately have all that build power to build another solar right in the same area. At least until you've completely covered the area with solars. And so it's okay to not overbuild that E. You don't need to.
[03:07] the user: this
[03:07] thebluegecko: Ah. Now here it's happening again. I metal stalling hard into a full E bar at 416.
[03:23] the user: was needed more uh con bots a bit ago see how many spend have okay
[03:33] thebluegecko: Probably not more constructors because
[03:40] thebluegecko: you want to have about one more constructor than your opponent has. And being tied is kind of okay. okay, having two more of them is kind of dangerous because that means they could have two more Blitzes than you. That's enough to overcome a Sentry.
[03:58] thebluegecko: So you wouldn't be able to win even a defensive battle, potentially, if you get down even just a little bit more.
[04:28] thebluegecko: This number of Sentries, the LLTs, is probably reasonable against bots because the LLTs are reasonably good against bots.
[04:41] thebluegecko: One way to think about LLTs is how much they force your opponent to build in order to fight you. You tend to want an LLT because otherwise a single rover can come kill all your mechs. So if you put an LLT next to the mex, then it takes two Blitz to raid that mex.
[05:09] thebluegecko: When you're playing against bots, it means you need five bots in order to raid that mex, so the cost value is just higher. When playing against a vehicle, that means you can get a little bit more value against players who aren't building rovers by skipping defenses on more mexes and putting two LLTs. on some, because the raiding group is likely to run into them and get killed that way.
[05:39] the user: yeah
[05:40] thebluegecko: But that does require your opponent to APM stall. If your opponent is not APM stalling and is just really good at the game, then that stops working, and the only possible defense becomes your own blitzes.
[05:58] thebluegecko: And so, between two really good players, the LLTs are not built as much. just using the units for defense tends to be stronger because they can move around. And you can kind of block that way. This does make the game more volatile because the first player to get a bunch of Blitzes into the undefended Solars is very likely to win. So it makes for a violent, harsh, nasty game.
[06:28] thebluegecko: And we're seeing some of that happen actually in this game here. At 730, the Blitzes get into the Solars and start dying.
[06:31] the user: i'm
[06:37] the user: in sync and
[06:41] the user: our our economy is just slowly ablating and now we're behind that
[06:46] thebluegecko: Yeah, so U6 got a small economic advantage advantage and then won a fight and got a big bigger economic advantage and then that just spiraled.
[07:03] thebluegecko: U6 was up just a couple of Blitzes and then winning meant up a few more and then a few more and on it goes. So at this point it's time for something desperate. The
[07:18] thebluegecko: Blitzes can't be used defensively anymore. Pretty much have to send them to the side of the map and try to start start picking off mechs. You know that sitting back while down metal income will eventually result in a loss. Every second that passes now puts blue more and more ahead. So red needs to do something desperate.
[07:45] thebluegecko: Probably medium tanks are no longer a good choice. I see one medium tank, one blitz. This only works if you're going to raid with the blitz. also probably want to get the units that are in the main out. When you get to a point where things are desperate, the units need to all be engaged. You can sometimes find the advantage you need by causing your opponent's units to not be used while yours are used. So even now, the army value now has a 15% blue lead, which is a pretty big deal here, 21% now. If the blue units are chasing red units but not actually firing, the red units might be doing damage for free. So try to get the fight on the other side of the map as soon as possible, which would also let you pick up the nearly 2k metal laying on the ground.
[08:49] the user: is something i need need to figure out how to teach the bot the uh the value of determining where your enemy where the enemy's army is by punching them in the face yeah
[08:59] thebluegecko: Yes, and the anti-valley pickings of that metal. Right now the metal produced shows blue ahead 18%, 14k to 11k. The metal on the ground, if successfully picked up, would not quite, but almost make that even again.
[09:19] the user: and
[09:31] the user: past the death past this point it's really just uh deaths uh the uh the death sprawl of uh doing something without being able to do anything for
[09:50] thebluegecko: Probably time to eat one of the two labs, too. Only 20 metal per second, you can't really run two labs. This solar that's currently building is a mistake as well. We're not spending all of our E. The E storage was great. That's a way to get some use out of it. I really
[10:22] thebluegecko: need to take the mex that is in the main and not taken. There's a constructor right there. It's protected. I
[10:35] the user: those initial battles at like five minutes and before would you think that the uh the medium tanks at all are a mistake that you should just focus on blitzes or that
[10:49] thebluegecko: want to be a little bit careful with that answer. I do think Blitzes were stronger there.
[10:58] thebluegecko: But it's not a true universal. On this map especially, the Blitzes are going to stay strong for a long time. And Blitzes trade kind of reasonably as long as they can get in amongst the medium tanks. And they're fast enough to outrun them. And so anytime you've got two army blobs moving, moving, the Blitz player is probably going to get around the medium tanks. Medium tanks can kind of be used defensively, but the speed just kind of matters enough.
[11:32] thebluegecko: Can I add here, to stop the Blitzes, the Janus makes more sense than the medium tanks. In small numbers, the Blitz can actually dodge the medium tank shots, juke back and forth and drive around the plasma. I
[12:05] thebluegecko: think the not transitioning to T2 is correct. That is the correct choice. Glad not to see that. The E-converters are actually a fairly serious mistake.
[12:19] thebluegecko: If you are winning, you might choose to do that for efficiency reasons, just temporarily. But if you're losing, what you need most, more than anything else, is tempo. You're already down metal, so you're down the main win condition other than killing the commander. commander. So what you need is some way to get on the other side of the map, start doing damage, or pick up a big metal pile or something. And what you need there is time. So if you've got E that's wasting, eat a solar. Eating a solar wastes build power, but it is fully metal efficient. So you can, if you're wasting E, just eat the solar, drop that metal into more tanks, and try to make something happen tempo-wise. Kill some Xs, draw the army off. off. Small groups of Blitzes would be really effective on the other side of the map right now. Almost all the enemy mechs are protected by just one LLT. So if you were hoping to raid past five LLTs, then assuming you lose one Blitz per LLT, send something like seven or eight Blitz, you can go through and raid through all of that and come out pretty strongly strongly positive.
[13:35] the user: reminds me i need to make sure that rove works with groups of units because yeah that i
[13:40] u6bkep: think you need a separate mode. Rove and raid are separate modes of operation. I
[13:46] the user: mean i'm looking at your map here and thinking if rove just took three or four uh blitzes and then used the exact same algorithm just with the ability to murder sentries which it already can do and it has the uh uh the army advantage right
[14:05] thebluegecko: We're looking at a serious error here, too. Red's army value is half of Blue's, but Red is defensive and may be able to bring their entire army value to bear. Well, Blue can only bring half of theirs because of the distance. This is a chance for Red, where the army is on Red's side of the map, the metal pile will be controlled by Red, but a bunch of red's units are behind the main. So
[14:36] u6bkep: will face tank with my main. Protect the units.
[14:43] u6bkep: This is also how I end up with my army trapped behind a large pile of bodies and fighting a superior force because I roll up to the main and there's no units and I do some damage. And then twice my army value in medium tanks rolls out from behind.
[15:03] thebluegecko: then the next thing going wrong here, trying to move the units through the main is causing them to filter out, and so now blue is fighting with a concave. I know it's just straight-up line, but 20 blue tanks are firing at 4 to 5 red tanks at a time, and the exchange of fire is not equal.
[15:21] the user: would
[15:37] thebluegecko: It'll be tempting for Red to think, well, I see blue tanks now below my base, that's why I had tanks down there. But it doesn't work. You've got to try to isolate these armies and kill one of them so you can harvest it and then kick the metal back into units. Because every second puts Red further behind now.
[16:00] thebluegecko: That metal advantage used to be 20%, now it's 58%.
[16:06] the user: you have been trying to build an air lab at any point in here well
[16:10] thebluegecko: Never had the E for it.
[16:19] thebluegecko: If I was blue, I would have.
[16:23] the user: okay Okay, yes. But, well, blue is designed to go for the BM, uh, AVP.
[16:24] thebluegecko: But I
[16:29] thebluegecko: do see the T2 plant here. In some ways, you really can't blame Blue for this. It's slower, it's a little rude, but it does...
[16:29] u6bkep: Thank
[16:40] thebluegecko: There are ways to lose a game for Blue, and one of them is lose a whole metal pile and then not be far enough ahead. Blue is far enough ahead to stop that, I think, but putting up some T2 mechs would mean being so far ahead that even if Red were to eat all of these units while still alive and they didn't even fire back, Red would still be behind. So this is just a very, very safe play, and
[17:16] thebluegecko: a few Mousers and a Prophet would make the base really easy to open up.
[17:24] thebluegecko: Knowing Blue, though, there's some risk he'll just go straight for Bulls.
[17:39] thebluegecko: I love that there's still a Rover out looking around for damage. It's really good play to have a Rover out looking for damage even this late. It's a really cheap way to do damage if there's even the smallest hole. And from the observer's eye view, which I know isn't something available to the players, there are eight or nine mechs that that rover could kill. It would be great value.
[18:02] the user: Yeah, it needs to have a bit more prioritization. It's doing an awful lot of half damage to, uh, structures. This
[18:14] thebluegecko: And absorbing human player APM that our bot is faster than human player is an advantage that it's great to take advantage of.
[18:23] thefiregecko: Ben, this is a Blitz, by the way.
[18:26] u6bkep: you. Which
[18:32] u6bkep: means I probably actually want the quasar cannon.
[18:43] thebluegecko: Oh, Mouser. Go ban.
[18:44] the user: is basically...
[18:47] thebluegecko: That is the right unit choice.
[18:53] thebluegecko: Another right choice would have been before building any V2 mechs, maybe one, but but don't build more. Get that fusion up, because having enough E for the fusion is hard.
[19:06] u6bkep: Yeah, the fusion goes up next. Does not take me long to realize that.
[19:08] thebluegecko: And it is really easy to forget just how much E those D2 mechs take. They're minus 20 E per second each.
[19:27] thebluegecko: And 7,000 E each, too.
[19:28] u6bkep: Coordination failure.
[19:52] thebluegecko: Red should definitely try to kill that lower army. Try and break out. Eat the metal pile.
[20:08] the user: should it be building more or less pure Janus here?
[20:16] thebluegecko: Probably the choice at this point is just to surrender. This is really, really bad.
[20:21] the user: Yes.
[20:21] u6bkep: Is this after I got the surround on the main? Oh,
[20:24] thebluegecko: After you've had it for quite some time.
[20:26] u6bkep: yeah. There's no coming back from that.
[20:29] the user: It actually does surrender.
[20:32] u6bkep: But not until the Mausers start firing on it.
[20:41] thebluegecko: 20 mousers, good heavens.
[20:48] the user: Alright, anything else before I stop the recording?
[20:52] thebluegecko: No, I don't think so.
