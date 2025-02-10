<script setup lang="ts">

import WxLogin from "./component/WxLogin.vue";
import WxMember from "./component/WxMember.vue";
import WxRule from "./component/WxRule.vue";

import {globalStore} from "./stores/global_store.ts";
import {storeToRefs} from "pinia";
import WxUser from "./component/WxUser.vue";
import WxBan from "./component/WxBan.vue";

const store = globalStore()
const {loading_completed, allow} = storeToRefs(store)
</script>
<template>
  <div class="main">
    <WxLogin v-if="allow&&!loading_completed" id="wx-login"/>
    <div class="container">
      <div class="left">
        <WxUser v-if="allow&&loading_completed" id="wx-user"/>
        <WxMember v-if="allow&&loading_completed" id="wx-member"/>
      </div>
      <div class="right">
        <WxRule v-if="allow&&loading_completed" id="wx-rule"/>
      </div>
    </div>
    <WxBan id="wx-ban" v-if="!allow"></WxBan>
  </div>
</template>

<style scoped>

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

#wx-login {
  width: 100vw;
  height: 100vh;
}

.main {
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
}

</style>