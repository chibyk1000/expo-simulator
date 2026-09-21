import React from "react";
import {
  View,
  Text,
  Pressable,
  TextInput,
  ScrollView,
  Image,
  ImageBackground,
} from "react-native";

export default function App() {
  console.log("🚀 Initialized App with Image, Network & Inspector features");

  const handleNetworkFetch = () => {
    console.log("🌐 Initiating fetch to https://api.github.com/zen...");
    if (typeof fetch !== "undefined") {
      fetch("https://api.github.com/zen")
        .then((res) => {
          console.log("✅ Fetch response received status:", res.status);
          return res.text();
        })
        .then((data) => {
          console.log("💬 GitHub Zen:", data);
        })
        .catch((err) => {
          console.warn("⚠️ Fetch error:", String(err));
        });
    } else {
      console.warn("Fetch polyfill not found");
    }
  };

  return (
    <ScrollView
      className="flex-1 bg-slate-900"
      contentContainerStyle={{
        padding: 20,
      }}
    >
      {/* Keep clear of the status bar / Dynamic Island */}
      <View style={{ height: 48 }} />
      {/* App Header with Local Image Asset */}
      <View className="flex-row items-center mb-5">
        <Image
          source="assets/expo-icon.png"
          style={{
            width: 48,
            height: 48,
            borderRadius: 12,
            marginRight: 14,
            backgroundColor: "#1e293b",
          }}
        />
        <View className="flex-1">
          <Text className="text-white text-2xl font-bold">Expo Simulator</Text>
          <Text className="text-cyan-400 text-xs font-semibold">
            React Native Native Runtime • v1.0
          </Text>
        </View>
      </View>

      {/* ImageBackground Hero Card */}
      <ImageBackground
        source="card-gradient.png"
        resizeMode="cover"
        style={{
          borderRadius: 16,
          padding: 16,
          marginBottom: 16,
          backgroundColor: "#1e293b",
          borderColor: "#334155",
          borderWidth: 1,
        }}
      >
        <Text className="text-white text-base font-bold mb-1">
          Image & Background Support
        </Text>
        <Text className="text-slate-300 text-xs mb-3">
          TinySkia procedural landscape renderer with resizeMode cover and rounded mask clipping.
        </Text>
        <View className="flex-row gap-2">
          <Image
            source="badge.png"
            resizeMode="contain"
            style={{ width: 64, height: 28, borderRadius: 6 }}
          />
        </View>
      </ImageBackground>

      {/* Network Telemetry Trigger Button */}
      <Pressable
        onPress={handleNetworkFetch}
        className="bg-cyan-600 p-4 rounded-xl mb-4 items-center"
      >
        <Text className="text-white font-bold text-sm">
          ⚡ Trigger Network Request (fetch)
        </Text>
      </Pressable>

      {/* Interactive Text Input */}
      <TextInput
        placeholder="Type here to test keyboard input..."
        placeholderTextColor="#94A3B8"
        className="bg-slate-800 text-white p-4 rounded-xl mb-4 border border-slate-700"
      />

      {/* Dev Menu & Inspector Hint Card */}
      <View className="bg-slate-800/80 p-4 rounded-xl border border-slate-700/60 mb-4">
        <Text className="text-slate-300 text-xs font-semibold mb-1">
          🛠️ Developer Tools Shortcuts
        </Text>
        <Text className="text-slate-400 text-xs">
          • Ctrl+D / Cmd+D : Open Expo Dev Menu{"\n"}
          • Ctrl+I : Toggle Element Inspector{"\n"}
          • Ctrl+O : Rotate Device Orientation{"\n"}
          • Ctrl+T : Toggle Light / Dark Theme{"\n"}
          • Esc : Toggle Developer Console Drawer
        </Text>
      </View>
    </ScrollView>
  );
}
