<script setup lang="ts">

import WxLogin from "./component/WxLogin.vue";
import WxMember from "./component/WxMember.vue";
import WxRule from "./component/WxRule.vue";

import {globalStore} from "./stores/global_store.ts";
import {storeToRefs} from "pinia";
import WxUser from "./component/WxUser.vue";

const store = globalStore()
const {homepage_enable} = storeToRefs(store)
</script>
<template>
  <div class="main">
    <WxLogin v-if="!homepage_enable" id="wx-login"/>
    <div class="container" v-if="homepage_enable">
      <div class="left">
        <WxUser id="wx-user"/>
        <WxMember id="wx-member"/>
      </div>
      <div class="right">
        <WxRule id="wx-rule"/>
      </div>
    </div>
  </div>
</template>

<style scoped>

.main {
  height: calc(var(--vh) * 100);
}

.container {
  display: flex;
  height: 100vh;
  width: 100vw;
}

.left {
  display: flex;
  width: 325px;
  height: 100vh;
  overflow: clip;
}

.right {
  flex: 1;
  display: flex;
  height: 100vh;
  overflow-y: auto;
}

#wx-ban {
  width: 100vw;
  height: 100vh;
}

#wx-user {
  width: 60px;
}

#wx-member {
  width: 265px;
}

</style>
<style>

body, html {
  margin: 0;
  padding: 0;
  font-family: "微软雅黑", serif;
  font-size: 12px;
  overflow: hidden;
}

</style>