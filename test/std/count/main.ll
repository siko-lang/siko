%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Int_Int = type { i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b16i4 = alloca %struct.Int_Int, align 8
   %b16i2 = alloca %struct.Int_Int, align 8
   %b16i1 = alloca %struct.Int_Int, align 8
   %b14i1 = alloca %struct.Int_Int, align 8
   %b11i1 = alloca %struct.Int_Int, align 8
   %b10i8 = alloca %struct.siko_Tuple_, align 4
   %b10i7 = alloca %struct.siko_Tuple_, align 4
   %b10i6 = alloca %struct.Bool_Bool, align 4
   %b10i4 = alloca %struct.Int_Int, align 8
   %b10i3 = alloca %struct.Int_Int, align 8
   %r_7 = alloca %struct.Int_Int, align 8
   %b10i1 = alloca %struct.Int_Int, align 8
   %match_var_6 = alloca %struct.Int_Int, align 8
   %b9i6 = alloca %struct.Bool_Bool, align 4
   %b9i4 = alloca %struct.Int_Int, align 8
   %b9i3 = alloca %struct.Int_Int, align 8
   %a_5 = alloca %struct.Int_Int, align 8
   %b9i1 = alloca %struct.Int_Int, align 8
   %b8i4 = alloca %struct.Int_Int, align 8
   %b8i2 = alloca %struct.Int_Int, align 8
   %b8i1 = alloca %struct.Int_Int, align 8
   %b6i1 = alloca %struct.Int_Int, align 8
   %loop_var_4 = alloca %struct.Int_Int, align 8
   %b2i8 = alloca %struct.Int_Int, align 8
   %b2i7 = alloca %struct.siko_Tuple_, align 4
   %b2i6 = alloca %struct.Bool_Bool, align 4
   %b2i4 = alloca %struct.Int_Int, align 8
   %b2i3 = alloca %struct.Int_Int, align 8
   %r_3 = alloca %struct.Int_Int, align 8
   %b2i1 = alloca %struct.Int_Int, align 8
   %match_var_2 = alloca %struct.Int_Int, align 8
   %b1i6 = alloca %struct.Bool_Bool, align 4
   %b1i4 = alloca %struct.Int_Int, align 8
   %b1i3 = alloca %struct.Int_Int, align 8
   %a_1 = alloca %struct.Int_Int, align 8
   %b1i1 = alloca %struct.Int_Int, align 8
   %loop_var_0 = alloca %struct.Int_Int, align 8
   %b0i1 = alloca %struct.Int_Int, align 8
   %tmp_b0i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i1, i32 0, i32 0
   store i64 1, ptr %tmp_b0i1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_0, ptr align 8 %b0i1, i64 8, i1 false)
   br label %block1
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b1i1, ptr align 8 %loop_var_0, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %a_1, ptr align 8 %b1i1, i64 8, i1 false)
   %tmp_b1i3_1 = getelementptr inbounds %struct.Int_Int, ptr %b1i3, i32 0, i32 0
   store i64 10, ptr %tmp_b1i3_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b1i4, ptr align 8 %a_1, i64 8, i1 false)
   call void @Int_Int_lessThan(ptr %b1i4, ptr %b1i3, ptr %b1i6)
   br label %block4
block2:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b2i1, ptr align 8 %loop_var_0, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %r_3, ptr align 8 %b2i1, i64 8, i1 false)
   %tmp_b2i3_1 = getelementptr inbounds %struct.Int_Int, ptr %b2i3, i32 0, i32 0
   store i64 10, ptr %tmp_b2i3_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b2i4, ptr align 8 %r_3, i64 8, i1 false)
   call void @Int_Int_eq(ptr %b2i4, ptr %b2i3, ptr %b2i6)
   call void @Std_Basic_Util_assert(ptr %b2i6, ptr %b2i7)
   %tmp_b2i8_1 = getelementptr inbounds %struct.Int_Int, ptr %b2i8, i32 0, i32 0
   store i64 1, ptr %tmp_b2i8_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_4, ptr align 8 %b2i8, i64 8, i1 false)
   br label %block9
block4:
   %tmp_switch_var_block4_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b1i6, i32 0, i32 0
   %tmp_switch_var_block4_2 = load i32, ptr %tmp_switch_var_block4_1, align 4
   switch i32 %tmp_switch_var_block4_2, label %block5 [
i32 1, label %block7
]

block5:
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b6i1, ptr align 8 %a_1, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_0, ptr align 8 %b6i1, i64 8, i1 false)
   br label %block2
block7:
   br label %block8
block8:
   %tmp_b8i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b8i1, i32 0, i32 0
   store i64 1, ptr %tmp_b8i1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b8i2, ptr align 8 %a_1, i64 8, i1 false)
   call void @Int_Int_add(ptr %b8i2, ptr %b8i1, ptr %b8i4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_0, ptr align 8 %b8i4, i64 8, i1 false)
   br label %block1
block9:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b9i1, ptr align 8 %loop_var_4, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %a_5, ptr align 8 %b9i1, i64 8, i1 false)
   %tmp_b9i3_1 = getelementptr inbounds %struct.Int_Int, ptr %b9i3, i32 0, i32 0
   store i64 10, ptr %tmp_b9i3_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b9i4, ptr align 8 %a_5, i64 8, i1 false)
   call void @Int_Int_lessThan(ptr %b9i4, ptr %b9i3, ptr %b9i6)
   br label %block12
block10:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b10i1, ptr align 8 %loop_var_4, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %r_7, ptr align 8 %b10i1, i64 8, i1 false)
   %tmp_b10i3_1 = getelementptr inbounds %struct.Int_Int, ptr %b10i3, i32 0, i32 0
   store i64 10, ptr %tmp_b10i3_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b10i4, ptr align 8 %r_7, i64 8, i1 false)
   call void @Int_Int_eq(ptr %b10i4, ptr %b10i3, ptr %b10i6)
   call void @Std_Basic_Util_assert(ptr %b10i6, ptr %b10i7)
   call void @siko_Tuple_(ptr %b10i8)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b10i8, i64 0, i1 false)
   ret void
block11:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b11i1, ptr align 8 %match_var_6, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_4, ptr align 8 %b11i1, i64 8, i1 false)
   br label %block9
block12:
   %tmp_switch_var_block12_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b9i6, i32 0, i32 0
   %tmp_switch_var_block12_2 = load i32, ptr %tmp_switch_var_block12_1, align 4
   switch i32 %tmp_switch_var_block12_2, label %block13 [
i32 1, label %block15
]

block13:
   br label %block14
block14:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b14i1, ptr align 8 %a_5, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_4, ptr align 8 %b14i1, i64 8, i1 false)
   br label %block10
block15:
   br label %block16
block16:
   %tmp_b16i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b16i1, i32 0, i32 0
   store i64 1, ptr %tmp_b16i1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b16i2, ptr align 8 %a_5, i64 8, i1 false)
   call void @Int_Int_add(ptr %b16i2, ptr %b16i1, ptr %b16i4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_6, ptr align 8 %b16i4, i64 8, i1 false)
   br label %block11
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %b6i1 = alloca %struct.siko_Tuple_, align 4
   %b4i2 = alloca %struct.siko_Tuple_, align 4
   %b4i1 = alloca %struct.siko_Tuple_, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i1, ptr align 4 %v, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i1, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b0i1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %b4i1)
   call void @siko_Tuple_(ptr %b4i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b4i2, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @siko_Tuple_(ptr %b6i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b6i1, i64 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

declare void @Int_Int_add(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_lessThan(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


