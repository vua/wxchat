<script setup lang="ts">
import {convertFileSrc, invoke} from "@tauri-apps/api/core";
import {ref} from "vue";
import {globalStore, Member, User} from "../stores/global_store.ts";
import {storeToRefs} from "pinia";
import {ElLoading} from 'element-plus';
import 'element-plus/theme-chalk/el-loading.css'
import {Picture as IconPicture} from '@element-plus/icons-vue'
import * as path from '@tauri-apps/api/path';

const store = globalStore()
const {homepage_enable, members, user, root, logout, expired_time} = storeToRefs(store)

let imgSrc = ref("");

async function getImgSrc() {
  root.value = await path.resourceDir();
  imgSrc.value = convertFileSrc(`${root.value}/resources/image/code/qr_code.jpg`)
}

getImgSrc()

const timestamp = ref(new Date().getTime());

async function refresh() {
  await invoke("refresh", {});
  timestamp.value = new Date().getTime()
  console.log("更新微信登录二维码")
}

async function login_check() {
  let code = await invoke("login_check", {}) as string;
  console.log("登录状态码:", code)
  return code
}

async function load_members() {
  members.value = JSON.parse(await invoke("get_members", {})) as Member[];
  console.log("加载微信联系人")
}

async function load_user() {
  user.value = JSON.parse(await invoke("get_user", {})) as User;
  console.log("加载个人信息")
}

async function load_auth() {
  expired_time.value = JSON.parse(await invoke("get_auth", {})) as number;
  console.log("加载权限信息")
}

async function sync_check() {
  console.log("开始执行 sync_check");
  try {
    const result = await Promise.race([
      invoke("sync_check", {}).then(res => {
        console.log("invoke 调用成功:", res);
        return res;
      }),
      new Promise((_, reject) => {
        setTimeout(() => {
          console.log("触发超时");
          reject(new Error('sync_check timeout'))
        }, 15000)
      })
    ]) as boolean;
    console.log("sync_check 执行完成:", result);
    return result;
  } catch (error) {
    console.log("sync_check 捕获到错误:", error);
    throw error;
  }
}

const sync_check_handler = function () {
  sync_check().then(async (status: boolean) => {
    if (!logout.value) { // 也没啥大用
      return
    }
    if (!status) {
      homepage_enable.value = false;
    } else {
      setTimeout(sync_check_handler, 1000);
    }
  }).catch(error => {
    console.log(`sync_check 请求失败:`, error);
    setTimeout(sync_check_handler, 10000);
  });
}


refresh().then(async () => {
  console.log("登录页面加载完成")
  login_check_handler()
})

const login_check_handler = function () {
  login_check().then(async (code: string) => {
    if (code == "200") {
      let loading_instance = ElLoading.service({
        lock: true,
        text: "加载中",
        background: 'rgba(0, 0, 0, 0.7)'
      })
      await load_auth()
      await load_user()
      await load_members()
      loading_instance.close()
      homepage_enable.value = true
      sync_check_handler()
    } else {
      if (code == "400") {
        refresh().then(async () => {
          login_check_handler()
        })
        return
      }
      setTimeout(login_check_handler, 1000)
    }
  }).catch(error => {
    console.log('请求异常，10秒后重试:', error);
    setTimeout(login_check_handler, 10000); // 失败后延长重试间隔
  });
}

</script>

<template>
  <div class="login-container">
    <div class="banner-area">
<pre>
 __      __        _________ .__            __
/  \    /  \___  __\_   ___ \|  |__ _____ _/  |_
\   \/\/   /\  \/  /    \  \/|  |  \\__  \\   __\
 \        /  >    <\     \___|   Y  \/ __ \|  |
  \__/\  /  /__/\_ \\______  /___|  (____  /__|
       \/         \/       \/     \/     \/

</pre>
      <pre style="font-size: 18px">你的微信自动回复助手</pre>

    </div>
    <div class="scan-area">
      <div style="margin: 5px">
        微信扫码登录
      </div>
      <div class="qr-code">
        <el-image :src="`${imgSrc}?timestamp=${timestamp}`">
          <template #error>
            <div class="image-slot">
              <el-icon>
                <IconPicture/>
              </el-icon>
            </div>
          </template>
        </el-image>
      </div>
    </div>
  </div>
</template>Ç

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: row;
  width: 100vw;
  height: 100vh;
}

.banner-area {
  background-image: linear-gradient(to right, #6DC1F7, #BCF1F2);
  color: cornsilk;
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: column;
  height: 100vh;
  flex: 5;
}

.scan-area {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: column;
  height: 100vh;
  flex: 4;
}

.qr-code {
  border: #ccc 1px solid;
  margin: 15px;
}

.qr-code .el-image {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 200px;
  height: 200px;
}

.qr-code .el-icon {
  font-size: 40px;
  color: #cccccc;
}
</style>